#![allow(warnings)]

static PROTO_VERSION: u8 = 1;

#[macro_export]
macro_rules! test_println {
    ($($arg:tt)*) => {

        #[cfg(test)]
        print!("[[ {} ]] ", <&str as colored::Colorize>::blue("TEST"));

        #[cfg(test)]
        print!($($arg)*);

        #[cfg(test)]
        print!("\n");
    };
}

#[macro_export]
macro_rules! test_print {
    ($($arg:tt)*) => {

        #[cfg(test)]
        print!("[[ {} ]] ", <&str as colored::Colorize>::blue("TEST"));

        #[cfg(test)]
        print!($($arg)*);
    };
}

// the core part has the code that can run on any platform
mod core {
    pub mod config;
    pub mod error;
    pub mod id;
    pub mod instance;
    pub mod item;
    pub mod macros;
    pub mod memstore;
    pub mod proto;
    pub mod types;
}

pub use async_trait::async_trait;
pub use core::error::MizeError;
pub use core::error::MizeResult;
pub use core::instance::module::Module;
pub use core::instance::Mize;
pub use core::instance::{
    DynMizePartGuard, MizePart, MizePartCreate, MizePartCreateGenerated, MizePartGenerated,
    MizePartGuard,
};
pub use core::*;
pub use mize_macros::*;
use std::path::PathBuf;

// platform specific stuff
pub mod platform {
    #[cfg(feature = "wasm-target")]
    pub mod wasm;

    #[cfg(feature = "os-target")]
    pub mod os;

    pub mod any {
        //////////// instance_init
        #[cfg(feature = "wasm-target")]
        pub use super::wasm::wasm_instance_init as instance_init;

        #[cfg(feature = "os-target")]
        pub use super::os::os_instance_init as instance_init;

        #[cfg(not(any(feature = "os-target", feature = "wasm-target")))]
        pub use super::super::instance_init;

        //////////// load_module
        #[cfg(feature = "os-target")]
        pub use super::os::load_module;

        #[cfg(feature = "wasm-target")]
        pub use super::wasm::load_module;

        #[cfg(not(any(feature = "os-target", feature = "wasm-target")))]
        pub use super::super::load_module;

        //////////// fetch_module
        #[cfg(feature = "os-target")]
        pub use super::os::fetch_module;

        #[cfg(feature = "wasm-target")]
        pub use super::super::fetch_module;

        #[cfg(not(any(feature = "os-target", feature = "wasm-target")))]
        pub use super::super::fetch_module;
    }
}

pub fn instance_init(instance: &mut core::instance::Mize) {}

pub fn load_module(
    instance: &mut core::instance::Mize,
    name: &str,
    path: Option<String>,
) -> MizeResult<()> {
    Ok(())
}

pub fn fetch_module(instance: &mut core::instance::Mize, name: &str) -> MizeResult<String> {
    Ok("oh noooooooooooooo, something went really really wrong, if this ends up in the executable....".to_owned())
}
