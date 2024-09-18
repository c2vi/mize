use std::ptr::NonNull;
use std::panic;
use web_sys::js_sys;
use crate::platform::wasm::js_sys::Function;
use web_sys::{WorkerOptions, WorkerType};
use web_sys::Worker;

use crate::instance::{self, Instance};
use crate::error::MizeResult;
use crate::{mize_err, Module};
use crate::MizeError;
use crate::core::item::IntoItemData;


// console_log macro
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}
macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}
//end of console_log macro


// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


// A function imitating `std::thread::spawn`.
// thanks to: https://www.tweag.io/blog/2022-11-24-wasm-threads-and-messages/
pub fn wasm_spawn(f: impl FnOnce() -> MizeResult<()> + Send + 'static) -> MizeResult<()> {
    

    // to get window.mize_worker_url, which is set by some js code, which gets "compiled" by a
    // bundler, which sets window.mize_worker_url to the url of ./npm_pkg/worker.js
    // I hate the js ecosystem....
    // those urls then become base64 data urls containing the js code...... i hate my life
    // import worker from './worker.js?worker' ..... does however crate a worker-hash.js file in
    // the dist dir...... 
    // this is then however a function, that creates the worker.... so put that at
    // window.mize_worker_fn and call that here


    //let js_fn_code = r#"
        //return window.mize_worker_url
    //"#;
    //let js_fn = Function::new_no_args(js_fn_code);
    //let res = js_fn.call0(&web_sys::wasm_bindgen::JsValue::NULL).unwrap();


    let window_obj = JsValue::from(web_sys::window().unwrap());
    let worker_fn: Function = js_sys::Reflect::get(&window_obj, &("mize_worker_fn".into())).unwrap().into();
    let worker: Worker = worker_fn.call0(&web_sys::wasm_bindgen::JsValue::NULL).unwrap().into();

    /*
    let worker_url_href = js_sys::Reflect::get(&worker_url_obj, &("href".into())).unwrap();
    let worker_url = worker_url_href.as_string().unwrap();

    let mut worker_options = WorkerOptions::new();
    worker_options.set_type(WorkerType::Module);
    let worker = web_sys::Worker::new_with_options(
        worker_url.as_str(),
        &worker_options
    ).map_err(|e| mize_err!("From JsValue: {:?}", e))?;
    */

    // Double-boxing because `dyn FnOnce` is unsized and so `Box<dyn FnOnce()>` is a fat pointer.
    // But `Box<Box<dyn FnOnce()>>` is just a plain pointer, and since wasm has 32-bit pointers,
    // we can cast it to a `u32` and back.
    let ptr = Box::into_raw(Box::new(Box::new(f) as Box<dyn FnOnce() -> MizeResult<()>>));
    let msg = js_sys::Array::new();
    // Send the worker a reference to our memory chunk, so it can initialize a wasm module
    // using the same memory.
    msg.push(&wasm_bindgen::memory());
    // Also send the worker the address of the closure we want to execute.
    msg.push(&JsValue::from(ptr as u32));
    worker.post_message(&msg);

    Ok(())
}

#[wasm_bindgen]
// This function is here for `worker.js` to call.
pub fn worker_entry_point(addr: u32) {
  // Interpret the address we were given as a pointer to a closure to call.
  let closure = unsafe { Box::from_raw(addr as *mut Box<dyn FnOnce()>) };
  (*closure)();
}


pub fn wasm_instance_init(instance: &mut Instance) -> MizeResult<()> {
    console_log!("Hello world from wasm_instance_init!!!!!!!!!!");

    Ok(())
}

#[wasm_bindgen]
pub struct JsInstance {
    inner: NonNull<Instance>,
}

/*
#[wasm_bindgen]
pub struct JsModule {
    inner: NonNull<Box<dyn Module + Send + Sync>>,
    //inner: *mut Mme,
}
*/

#[wasm_bindgen]
impl JsInstance {
    #[wasm_bindgen(constructor)]
    pub fn new() -> JsInstance {
        panic::set_hook(Box::new(console_error_panic_hook::hook));

        let instance = match Instance::new() {
            Ok(val) => val,
            Err(e) => {
                console_log!("Instance::new() failed with: {:?}", e);
                panic!()
            },
        };
        let mut js_instance = JsInstance { inner: NonNull::from(Box::leak(Box::new(instance))) };
        return js_instance;
    }

    #[wasm_bindgen]
    pub unsafe fn test(&mut self) -> () {
        console_log!("hiiiiiiiiiiiiiiii from test");
        self.inner.as_mut().set("0/config/hello", "hello world".into_item_data());
    }

    //#[wasm_bindgen]

}
