use deno_core::error::AnyError;
use mize::{Mize, MizePart, MizeResult};
use std::rc::Rc;

pub struct JsPart {
  mize: Mize,
}

impl JsPart {
    pub fn new(mize: Mize) -> Self {
        JsPart { mize }
    }
}

impl MizePart for JsPart {
    fn name(&self) -> &'static str {
        "js"
    }
    
    fn get_mize(&mut self) -> &mut Mize {
        &mut self.mize
    }

    fn init(&mut self, mize: &mut Mize) -> MizeResult<()> {
        println!("deno part init");
        Ok(())
    }
}

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
