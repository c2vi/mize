use deno_core::error::AnyError;
use deno_core::url::Url;
use deno_core::{extension, op2, JsRuntime, ModuleSpecifier, PollEventLoopOptions, RuntimeOptions};
use deno_core::{FastStaticString, FsModuleLoader};
use mize::async_trait;
use mize::instance::MizePartCreate;
use mize::{mize_part, Mize, MizePart, MizeResult};
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::rc::Rc;
use std::str::FromStr;
use tokio::runtime::Builder;
use tokio::task::{spawn_local, LocalSet};

mod glue_deno;

#[mize_part("js")]
#[derive(Default)]
pub struct JsPart {
    mize: Mize,
    closure_sender: Option<flume::Sender<Box<dyn FnOnce(&mut JsRuntime) -> MizeResult<()> + Send>>>,
    async_closure_sender: Option<flume::Sender<BoxClosure>>,
}

pub type BoxFuture<'a> = Pin<Box<dyn Future<Output = MizeResult<()>> + 'a>>;
pub type BoxClosure = Box<dyn for<'a> FnOnce(&'a mut JsRuntime) -> BoxFuture<'a> + Send + 'static>;

impl MizePart for JsPart {
    fn init(&mut self, mize: &mut Mize) -> MizeResult<()> {
        println!("js part init");
        Ok(())
    }
}

pub fn js(mize: &mut Mize) -> MizeResult<()> {
    let (closure_sender, closure_receiver) =
        flume::unbounded::<Box<dyn FnOnce(&mut JsRuntime) -> MizeResult<()> + Send>>();
    let (async_closure_sender, async_closure_receiver) = flume::unbounded::<BoxClosure>();
    let mize_clone = mize.clone();

    // the thread which will run any js
    mize.spawn("js_runtime_thread", || {
        js_runtime_thread(mize_clone, closure_receiver, async_closure_receiver)
    })?;

    mize.add_part(Box::new(JsPart {
        mize: mize.clone(),
        closure_sender: Some(closure_sender),
        async_closure_sender: Some(async_closure_sender),
    }))
}

pub fn part_from_file(
    mize: &mut Mize,
    name: &'static str,
    code: FastStaticString,
) -> MizeResult<()> {
    let mut js = mize.get_part_native::<JsPart>("js")?;
    js.with_runtime_async(move |runtime: &mut JsRuntime| {
        let module_spec = ModuleSpecifier::from_str(name).unwrap();
        return Box::pin(async move {
            runtime
                .load_side_es_module_from_code(&module_spec, code)
                .await;
            Ok(())
        });
    });
    Ok(())
}

impl JsPart {
    /*
    pub fn part_from_js_file(
        &mut self,
        name: &'static str,
        path: &'static str,
    ) -> Box<dyn MizePart + Send + Sync> {
        Box::new(PartFromJsFileAdapter {
            mize: self.mize.clone(),
            name,
            js_file: path,
        })
    }
    */
    pub fn with_runtime<T: FnOnce(&mut JsRuntime) -> MizeResult<()> + Send + 'static>(
        &mut self,
        func: T,
    ) {
        self.closure_sender
            .as_mut()
            .unwrap()
            .send(Box::new(func))
            .unwrap();
    }
    pub fn with_runtime_async<F>(&mut self, func: F)
    where
        F: for<'a> FnOnce(&'a mut JsRuntime) -> BoxFuture<'a> + Send + 'static,
    {
        self.async_closure_sender
            .as_mut()
            .unwrap()
            .send(Box::new(func))
            .unwrap();
    }
}

/*
#[mize_part]
#[derive(Default)]
struct PartFromJsFileAdapter {
    mize: Mize,
    name: &'static str,
    js_file: &'static str,
}

#[async_trait]
impl MizePart for PartFromJsFileAdapter {
    fn deps(&self) -> &'static [&'static str] {
        &["js"]
    }
    fn name(&self) -> &'static str {
        self.name
    }
    async fn async_init(&mut self, mize: &mut Mize) -> MizeResult<()> {
        let js = mize.get_part_native::<JsPart>("js")?;
        let path = self.js_file;
        let js_code = format!(
            r#"
             import {{ opts, deps, create }} from "{path}";

             const isPromise = (value) => {{
               return !!value && (typeof value === 'object' || typeof value === 'function') && typeof value.then === 'function';
             }};

             // Call the async function
             let create_result = create(mize);
             if (isPromise(create_result)) {{
                 create_result = await create_result;
             }}
             const result = {{
                 opts: opts(mize),
                 deps: deps(mize),
                 create: create_result,
             }};

             result;
         "#
        );

        // Execute the script
        let result = js.runtime().execute_script("[main]", js_code)?;

        // Resolve the promise and run event loop
        let resolved_value = js.runtime().resolve_value(result).await?;

        // Run the event loop to completion
        js.runtime()
            .run_event_loop(PollEventLoopOptions::default())
            .await?;

        println!("Execution of part {} completed successfully", self.name());
        Ok(())
    }
    async fn async_run(&mut self, mize: &mut Mize) -> MizeResult<()> {
        let js = mize.get_part_native::<JsPart>("js")?;
        let path = self.js_file;
        let js_code = format!(
            r#"
              import {{ run }} from "{path}";

              const result = run()

              result;
            "#
        );
        // Execute the script
        let result = js.runtime().execute_script("[main]", js_code)?;

        // Resolve the promise and run event loop
        let resolved_value = js.runtime().resolve_value(result).await?;

        // Run the event loop to completion
        js.runtime()
            .run_event_loop(PollEventLoopOptions::default())
            .await?;

        println!("Running of part {} completed successfully", self.name());
        Ok(())
    }
}
*/

fn js_runtime_thread(
    mize: Mize,
    closure_receiver: flume::Receiver<Box<dyn FnOnce(&mut JsRuntime) -> MizeResult<()> + Send>>,
    async_closure_receiver: flume::Receiver<BoxClosure>,
) -> MizeResult<()> {
    let mut js_runtime = JsRuntime::new(RuntimeOptions {
        module_loader: Some(Rc::new(FsModuleLoader)),
        extensions: vec![glue_deno::my_extension::init(mize.clone())],
        ..Default::default()
    });

    let poll_opts = PollEventLoopOptions::default();

    let tokio_runtime = Builder::new_current_thread().enable_all().build().unwrap();

    let local = LocalSet::new();
    local.block_on(&tokio_runtime, async move {
        loop {
            // Drain queued closures quickly (no await here if possible)
            if let Ok(func) = closure_receiver.recv_async().await {
                if let Err(e) = func(&mut js_runtime) {
                    mize.report_err(e);
                }
            }

            // Drive JS one tick
            let _ = futures::future::poll_fn(|cx| js_runtime.poll_event_loop(cx, poll_opts)).await;
        }
    });

    Ok(())
}
