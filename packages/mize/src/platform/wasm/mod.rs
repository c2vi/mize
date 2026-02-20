use core::convert::From;
use core::option::Option;
use std::ptr::NonNull;
use std::panic;
use web_sys::js_sys::{self, eval, Promise};
use crate::id::MizeId;
use crate::item::{Item, ItemData};
use crate::platform::wasm::js_sys::Function;
use web_sys::{WorkerOptions, WorkerType};
use web_sys::Worker;

use crate::instance::{self, Instance};
use crate::error::MizeResult;
use crate::{mize_err, Module};
use crate::MizeError;
use crate::core::item::IntoItemData;
use crate::instance::module::EmptyModule;




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



#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


pub fn wasm_instance_init(instance: &mut Instance) -> MizeResult<()> {
    console_log!("Hello world from wasm_instance_init!!!!!!!!!!");

    Ok(())
}

pub fn load_module(instance: &mut Instance, name: &str, path: Option<String>) -> MizeResult<()> {
    console_log!("loading module: {}", name);

    wasm_bindgen_futures::spawn_local(load_module_async(instance.clone(), name.to_owned(), path));

    Ok(())
}

async fn load_module_async(mut instance: Instance, name: String, path: Option<String>) -> () {

    let name = name.as_str();
    let module_url = fetch_module(&mut instance, name).unwrap();
    let memory = wasm_bindgen::memory();

    let mut empty_module: Box<dyn Module + Send + Sync> = Box::new(EmptyModule {});
    let mut mize = instance.clone();

    let empty_module_ptr = Box::into_raw(Box::new(empty_module));
    let empty_module_usize = empty_module_ptr as usize;
    let mize_usize = Box::into_raw(Box::new(mize)) as usize;

    console_log!("testttttttttttttt in load_module_async...");

    let function_str = format!(r#"
        const promise = new Promise((resolve, reject) => {{
            import("{module_url}/mize_module_{name}.js").then( module => {{
                const wasm_bindgen = module.get_wasm_bindgen();

                const {{ wasm_get_mize_module_mme }} = wasm_bindgen;

                wasm_bindgen("{module_url}/mize_module_{name}_bg.wasm", memory).then( ()  => {{
                    console.log("empty_module_usize:", empty_module_usize)
                    console.log("mize_usize:", mize_usize)
                    //resolve()
                    resolve(wasm_get_mize_module_mme(empty_module_usize, mize_usize))
                }})
            }})
        }})
        return promise;
    "#);


    let function = Function::new_with_args("memory, empty_module_usize, mize_usize", &function_str);

    let ret_value: JsValue = match function.call3(&JsValue::null(), &memory, &empty_module_usize.into(), &mize_usize.into()) {
        Ok(val) => val,
        Err(err) => {
            console_log!("failed to load the wasm mize module....");
            console_log!("{:?}", err);
            return;
        },
    };

    console_log!("got ret_value: {:?}", ret_value);
    let ret_promise = Promise::from(ret_value);
    let ret_result = match wasm_bindgen_futures::JsFuture::from(ret_promise).await {
        Ok(val) => val,
        Err(err) => {
            console_log!("err awaiting ret_promise");
            console_log!("{:?}", err);
            return;
        },
    };

    let mut filled_module = unsafe {
        Box::from_raw(empty_module_ptr)
    };

    console_log!("before....");
    let a = instance.get("0/config");
    console_log!("after....");

    //filled_module.init(&instance);


    /*
    console_log!("got ret_result: {:?}", ret_result);
    let ret_f64 = ret_result.as_f64().unwrap();
    console_log!("got ret_f64: {:?}", ret_f64);

    let ret_usize = ret_f64.floor() as usize;

    let mut module: Box<dyn Module + Send + Sync> = Box::new(EmptyModule {});

    unsafe {
        let get_mize_module_fn_ptr = ret_usize as *const fn(&mut Box<dyn Module + Send + Sync>, Instance) -> ();
        let get_mize_module_fn: fn(&mut Box<dyn Module + Send + Sync>, Instance) -> () = unsafe { std::mem::transmute(get_mize_module_fn_ptr) };
        get_mize_module_fn(&mut module, instance.clone());
    }
    */



}

pub fn fetch_module(instance: &mut Instance, name: &str) -> MizeResult<String> {

    let module_name_with_slashes = name.replace(".", "/");
    if let Ok(module_path) = instance.get(format!("self/config/module_dir/{}", module_name_with_slashes))?.value_string() {
        return Ok(module_path);

    } else {
        let module_url = instance.get("self/config/module_url")?.value_string()?;
        return Ok(format!("{}/{}", module_url, module_name_with_slashes));
    }
}

async fn download_module(instance: &mut Instance, name: &str) -> MizeResult<js_sys::ArrayBuffer> {
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, RequestInit, RequestMode, Response};

    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);

    let url = format!("{}/mize_module_{}_bg.wasm", fetch_module(instance, name)?, name);

    let request = Request::new_with_str_and_init(&url, &opts).unwrap();

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await.unwrap();

    let resp: Response = resp_value.dyn_into().unwrap();

    let wasm_bytes: js_sys::ArrayBuffer = JsFuture::from(resp.array_buffer().unwrap()).await.unwrap().dyn_into().unwrap();

    Ok(wasm_bytes)
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
pub async fn new_js_instance(config_json_str: String) -> JsInstance {

    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let config = ItemData::from_json(config_json_str).expect("parsing of json config failed");

    let mut instance = Instance::empty().expect("Instance::empty() failed");

    instance.set_blocking("0/config", config).expect("Failed to set the config at item 0");

    let mut js_instance = JsInstance { inner: NonNull::from(Box::leak(Box::new(instance))) };

    return js_instance;
}

#[wasm_bindgen]
impl JsInstance {

    #[wasm_bindgen]
    pub unsafe fn init(&mut self) -> MizeResult<()> {
        self.inner.as_mut().init()
    }

    #[wasm_bindgen]
    pub unsafe fn set(&mut self, id: String, value: String) -> () {
        let data = value.into_item_data();
        console_log!("data in set: {}", data);
        self.inner.as_mut().set(id, data);
    }

    #[wasm_bindgen]
    pub unsafe fn get_handle(&mut self, id: String) -> MizeResult<JsItemHandle> {
        let item = self.inner.as_mut().get(id)?;
        Ok(JsItemHandle { instance: self.inner, id: item.id()})
    }

}

impl JsInstance {
    pub fn inner(&mut self) -> &mut Instance {
        unsafe {
            self.inner.as_mut()
        }
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
