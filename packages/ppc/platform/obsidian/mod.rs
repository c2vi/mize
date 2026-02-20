use mize::Mize;
use wasm_bindgen::prelude::*;

// console_log macro

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => ({ log(&format_args!($($t)*).to_string())})
}

#[wasm_bindgen]
pub fn obsidian_mize_entrypoint() {
    console_log!("hiiiiiiiiiiiii from rust");
    let mize = Mize::new();
    console_log!("Obsidian Mize initialized");
}
