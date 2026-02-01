use base64::Engine;
use mize::{mize_err, MizeResult};
use mize::proto::MizeMessage;
use base64::engine::general_purpose::STANDARD;
use tracing::trace;
use ciborium::Value as CborValue;
use flume::{ Receiver, Sender };
use ciborium_io::Read;

#[cfg(feature = "os-target")]
use wry::http::Request;

#[cfg(feature = "os-target")]
use wry::WebView;

#[cfg(feature = "os-target")]
use tao::event_loop::EventLoopProxy;

#[cfg(feature = "wasm-target")]
use crate::implementors::html::wasm::MmeJs;

#[cfg(feature = "wasm-target")]
use web_sys::js_sys::Function;

#[cfg(feature = "wasm-target")]
use wasm_bindgen::JsValue;

 

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




use crate::mme::Mme;
use mize::MizeError;



#[cfg(feature = "os-target")]
pub fn mme_setup_weview_con_host(mme: &mut Mme, rx: Receiver<MizeMessage>, event_loop_proxy: EventLoopProxy<MizeMessage>) -> MizeResult<()> {

    let mize_clone = mme.mize.clone();
    mme.mize.spawn("webview-con-outgoing", move || {
        for msg in rx {
            event_loop_proxy.send_event(msg);
        }
        Ok(())
    });

    return Ok(());
}


#[cfg(feature = "wasm-target")]
pub fn mme_setup_webiew_con_js(mme: &mut Mme) -> MizeResult<Sender<MizeMessage>> {
    let (tx, rx): (Sender<MizeMessage>, Receiver<MizeMessage>) = flume::unbounded();

    // send things from rx out by calling window.ipc.postMessage("msg here as string")
    wasm_bindgen_futures::spawn_local(webview_con_js_send_out(mme.clone(), rx));

    let conn_id = mme.mize.new_connection_join_namespace(tx.clone())?;

    return Ok(tx);
}


#[cfg(feature = "wasm-target")]
pub async fn webview_con_js_send_out(mme: Mme, rx: Receiver<MizeMessage>) {
    let func = Function::new_with_args("msg_string", r#"
        fetch("/msg", { method: "POST", headers: { MizeMsg: msg_string}})
    "#);

    loop {
        let msg = match rx.recv_async().await {
            Ok(val) => val,
            Err(err) => { mme.mize.report_err(err.into()); continue; },
        };
        console_log!("webview_con_js_send_out: got_msg: {:?}", msg);

        let msg_string = match msg_to_string(msg) {
            Ok(val) => val,
            Err(err) => { mme.mize.report_err(err.into()); continue; },
        };

        func.call1(&JsValue::null(), &JsValue::from_str(msg_string.as_str()));
    }
}


pub fn msg_to_string(msg: MizeMessage) -> MizeResult<String> {

    let mut msg_bytes = Vec::new();

    ciborium::into_writer(&msg.value(), &mut msg_bytes)?;

    let msg_string = STANDARD.encode(msg_bytes);

    return Ok(msg_string)
}


pub fn msg_from_string(msg_string: String, conn_id: u64) -> MizeResult<MizeMessage> {

    let msg_bytes = STANDARD.decode(msg_string)?;

    let value: CborValue = ciborium::from_reader(msg_bytes.as_slice())?;

    let msg = MizeMessage::new(value, conn_id);

    return Ok(msg);
}



