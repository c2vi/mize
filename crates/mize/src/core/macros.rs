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

#[macro_export]
macro_rules! add_parts {
    ($mize:expr, $($part:expr),+) => {
        $(
          let part = $part.create($mize.clone());
          $mize.add_part(
                part
            );
        )+
        mize.init_parts();
    };
}
