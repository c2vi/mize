mod utils;

use wasm_bindgen::prelude::*;

use crate::instance::Instance;
use crate::error::MizeResult;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, wasm-game-of-life!");
}

pub fn wasm_instance_init(instance: &mut Instance) -> MizeResult<()> {
    println!("Hello world from wasm_instance_init!!!!!!!!!!");
    alert("Hello world from wasm_instance_init!!!!!!!!!!");

    Ok(())
}
