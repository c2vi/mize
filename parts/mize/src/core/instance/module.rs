// https://users.rust-lang.org/t/casting-between-trait-object-types/97220/2

use std::ffi::OsString;

use crate::error::MizeResult;
use crate::instance::Mize;

pub trait Module {
    fn init(&mut self, instance: &Mize) -> MizeResult<()>;

    fn exit(&mut self, instance: &Mize) -> MizeResult<()>;

    fn clone_module(&self) -> Box<dyn Module + Send + Sync>;

    // extra traits, that can be implemented
    // return None if not implemented

    fn run_cli(&mut self, instance: &Mize, cmd_line: Vec<OsString>) -> Option<MizeResult<()>> {
        None
    }
}

// console_log macro
// that can be copied into other files for debugging purposes
#[cfg(feature = "wasm-target")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm-target")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[cfg(feature = "wasm-target")]
macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (unsafe { log(&format_args!($($t)*).to_string())})
}

pub struct EmptyModule {}

impl Module for EmptyModule {
    fn init(&mut self, instance: &Mize) -> MizeResult<()> {
        println!("empty module fn init");

        #[cfg(feature = "wasm-target")]
        console_log!("empty module fn init");
        Ok(())
    }
    fn exit(&mut self, instance: &Mize) -> MizeResult<()> {
        println!("empty module fn exit");
        Ok(())
    }

    fn clone_module(&self) -> Box<dyn Module + Send + Sync> {
        Box::new(EmptyModule {})
    }
}
