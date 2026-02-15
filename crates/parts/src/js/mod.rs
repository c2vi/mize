use deno_core::FsModuleLoader;
use deno_core::error::AnyError;
use deno_core::{JsRuntime, ModuleSpecifier, PollEventLoopOptions, RuntimeOptions, extension, op2};
use mize::async_trait;
use mize::instance::MizePartCreate;
use mize::{Mize, MizePart, MizeResult, mize_part};
use std::path::Path;
use std::rc::Rc;

#[mize_part("js")]
pub struct JsPart {
    mize: Mize,
    js_runtime: JsRuntime,
}

impl MizePart for JsPart {
    fn init(&mut self, mize: &mut Mize) -> MizeResult<()> {
        println!("js part init");
        Ok(())
    }
}
impl MizePartCreate for JsPart {
    fn create(mize: &mut Mize) -> Box<dyn MizePart> {
        let mut js_runtime = JsRuntime::new(RuntimeOptions {
            module_loader: Some(Rc::new(FsModuleLoader)),
            extensions: vec![mize_ext::init_ops()],
            ..Default::default()
        });
        let bootstrap_code = r#"
             globalThis.mize = {
                 get_part: (name) => Deno.core.ops.op_mize_get_part(name),
                 get_config: (key) => Deno.core.ops.op_mize_get_config(key),
                 version: "1.0.0",
                 // Add more methods as needed
             };
         "#;

        js_runtime.execute_script("[bootstrap]", bootstrap_code)?;
        JsPart { mize, js_runtime }
    }
}

impl JsPart {
    pub fn part_from_js_file(
        &mut self,
        name: &'static str,
        path: &str,
    ) -> Box<dyn MizePart + Send + Sync> {
        Box::new(PartFromJsFileAdapter {
            mize: self.mize.clone(),
            name,
            js_file: path,
        })
    }
}

#[mize_part]
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
        let js: JsPart = mize.get_part_native("js");
        let path = self.js_file;
        let js_code = format!(
            r#"
             import {{ opts, deps, create }} from "{path}";
             
             const isPromise = (value) => {
               return !!value && (typeof value === 'object' || typeof value === 'function') && typeof value.then === 'function';
             };
             
             // Call the async function
             let create_result = create(mize);
             if (isPromise(create_result)) {
                 create_result = await create_result;
             }
             const result = {
                 opts: opts(mize),
                 deps: deps(mize),
                 create: create_result,
             };
             
             result;
         "#
        );

        // Execute the script
        let result = js.runtime.execute_script("[main]", js_code)?;

        // Resolve the promise and run event loop
        let resolved_value = js.runtime.resolve_value(result).await?;

        // Run the event loop to completion
        js.runtime
            .run_event_loop(PollEventLoopOptions::default())
            .await?;

        println!("Execution of part {} completed successfully", self.name());
        Ok(())
    }
    async fn async_run(&mut self, mize: &mut Mize) -> MizeResult<()> {
        let js: JsPart = mize.get_part_native("js");
        let path = self.js_file;
        let js_code = format!(
            r#"
              import {{ run }} from "{path}";
              
              const result = run()
              
              result;
            "#
        );
        // Execute the script
        let result = js.runtime.execute_script("[main]", js_code)?;

        // Resolve the promise and run event loop
        let resolved_value = js.runtime.resolve_value(result).await?;

        // Run the event loop to completion
        js.runtime
            .run_event_loop(PollEventLoopOptions::default())
            .await?;

        println!("Running of part {} completed successfully", self.name());
        Ok(())
    }
}

/*
async fn run_js_file(file_path: &str) -> Result<(), AnyError> {
    let main_module = deno_core::resolve_path(file_path, &std::env::current_dir()?)?;
    let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
        module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
        ..Default::default()
    });

    let mod_id = js_runtime.load_main_es_module(&main_module).await?;
    let result = js_runtime.mod_evaluate(mod_id);
    js_runtime.run_event_loop(Default::default()).await?;
    result.await
}
 */

// Define your custom ops for the "mize" object inside deno
#[op2(fast)]
fn op_mize_get_part(#[string] name: &str) {
    println!("js wants the part {}", name);
}

#[op2]
#[string]
fn op_mize_get_config(#[string] key: &str) -> String {
    format!("config_value_for_{}", key)
}

// Create an extension with your custom ops
extension!(mize_ext, ops = [op_mize_get_part, op_mize_get_config]);
