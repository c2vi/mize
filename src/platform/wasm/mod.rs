use std::ptr::NonNull;
use std::panic;
use web_sys::js_sys::{self, eval};
use crate::id::MizeId;
use crate::item::Item;
use crate::platform::wasm::js_sys::Function;
use web_sys::{WorkerOptions, WorkerType};
use web_sys::Worker;

use crate::instance::{self, Instance};
use crate::error::MizeResult;
use crate::{mize_err, Module};
use crate::MizeError;
use crate::core::item::IntoItemData;




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

#[cfg(not(feature = "wasm-target"))]
macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => ()
}
//end of console_log macro



// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


pub fn wasm_instance_init(instance: &mut Instance) -> MizeResult<()> {
    console_log!("Hello world from wasm_instance_init!!!!!!!!!!");

    eval("window.mize.mod = {}").map_err(|_| mize_err!("failed to set window.mize.mod to an empty object"))?;

    Ok(())
}

pub fn load_module(instance: &mut Instance, name: &str, path: Option<String>) -> MizeResult<()> {
    console_log!("loading module: {}", name);
    Ok(())
}

#[wasm_bindgen]
pub struct JsInstance {
    inner: NonNull<Instance>,
}


#[wasm_bindgen]
pub struct JsItemHandle {
    instance: NonNull<Instance>,
    id: MizeId,
}


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
    pub unsafe fn set(&mut self, id: String, value: String) -> () {
        let data = value.into_item_data();
        console_log!("data in set: {}", data);
        self.inner.as_mut().set_blocking(id, data);
    }

    #[wasm_bindgen]
    pub unsafe fn get_handle(&mut self, id: String) -> MizeResult<JsItemHandle> {
        let item = self.inner.as_mut().get(id)?;
        Ok(JsItemHandle { instance: self.inner, id: item.id()})
    }
}


#[wasm_bindgen]
impl JsItemHandle {

    #[wasm_bindgen]
    pub unsafe fn value_string(&mut self) -> MizeResult<String> {
        let item = self.instance.as_mut().get(self.id.clone())?;
        let string = item.value_string()?;
        Ok(string)
    }

    #[wasm_bindgen]
    pub unsafe fn as_data_full(&mut self) -> MizeResult<JsValue> {
        let item = self.instance.as_mut().get(self.id.clone())?;
        let data_raw = item.as_data_full()?;
        let jsvalue = serde_wasm_bindgen::to_value(data_raw.cbor())?;
        Ok(jsvalue)
    }

}


impl From<MizeError> for JsValue {
    fn from(value: MizeError) -> Self {
        let string = format!("{:?}", value);
        string.into()
    }
}
