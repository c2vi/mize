use deno_core::error::AnyError;
use deno_core::url::Url;
use deno_core::v8::OneByteConst;
use deno_core::{
    ascii_str_include, extension, op2, JsRuntime, ModuleSpecifier, PollEventLoopOptions,
    RuntimeOptions,
};
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
    sender: Option<flume::Sender<JsRuntimeThreadMessage>>,
}

pub type BoxFuture<'a> = Pin<Box<dyn Future<Output = MizeResult<()>> + 'a>>;
pub type BoxClosure = Box<dyn for<'a> FnOnce(&'a mut JsRuntime) -> BoxFuture<'a> + Send + 'static>;

impl MizePart for JsPart {
    fn run(&mut self, mize: &mut Mize) -> MizeResult<()> {
        println!("js part run");
        self.sender
            .as_ref()
            .unwrap()
            .send(JsRuntimeThreadMessage::DoRunPhase)
            .unwrap();
        Ok(())
    }
}

pub fn js(mize: &mut Mize) -> MizeResult<()> {
    let (sender, receiver) = flume::unbounded::<JsRuntimeThreadMessage>();
    let mize_clone = mize.clone();

    // the thread which will run any js
    mize.spawn_and_wait("js_runtime_thread", || {
        js_runtime_thread(mize_clone, receiver)
    })?;

    mize.add_part(Box::new(JsPart {
        mize: mize.clone(),
        sender: Some(sender),
    }))
}

pub fn part_from_file(
    mize: &mut Mize,
    name: &'static str,
    code: FastStaticString,
) -> MizeResult<()> {
    let mut js = mize.get_part_native::<JsPart>("js")?;
    js.execute_init_js(code)?;
    Ok(())
}

impl JsPart {
    pub fn execute_init_js(&mut self, code: FastStaticString) -> MizeResult<()> {
        self.sender
            .as_ref()
            .unwrap()
            .send(JsRuntimeThreadMessage::RunInitJs(code))
            .unwrap();
        Ok(())
    }
    pub fn eval(&mut self, code: String) -> MizeResult<()> {
        self.sender
            .as_ref()
            .unwrap()
            .send(JsRuntimeThreadMessage::RunJs(code))
            .unwrap();
        Ok(())
    }
}

fn js_runtime_thread(
    mize: Mize,
    receiver: flume::Receiver<JsRuntimeThreadMessage>,
) -> MizeResult<()> {
    let mut js_runtime = JsRuntime::new(RuntimeOptions {
        //module_loader: Some(Rc::new(FsModuleLoader)),
        //extensions: vec![glue_deno::my_extension::init(mize.clone())],
        ..Default::default()
    });

    js_runtime
        .execute_script("[stub]", ascii_str_include!("../deno_dist/glue_deno.js"))
        .unwrap();

    loop {
        println!("js thread waiting for smth");
        let msg = receiver.recv().unwrap();
        println!("js thread got smth");
        match msg {
            JsRuntimeThreadMessage::RunInitJs(js_code) => {
                if let Err(err) = js_runtime.execute_script("[init]", js_code) {
                    println!("err: {}", err);
                }
                println!("done running js");
            }
            JsRuntimeThreadMessage::RunJs(js_code) => {
                if let Err(err) = js_runtime.execute_script("[idk]", js_code) {
                    println!("err: {}", err);
                }
                println!("done running js");
            }
            JsRuntimeThreadMessage::DoRunPhase => {
                if let Err(err) = js_runtime.execute_script("[runPhase]", "mize.runPhase()") {
                    println!("err: {}", err);
                }
                println!("done with runPhase");
                return Ok(());
            }
        }
    }
}

enum JsRuntimeThreadMessage {
    //Closure(Box<dyn FnOnce(&mut JsRuntime) -> MizeResult<()> + Send>),
    //AsyncClosure(BoxClosure),
    RunInitJs(FastStaticString),
    RunJs(String),
    DoRunPhase,
}
