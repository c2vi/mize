
#![ allow( warnings ) ]

static PROTO_VERSION: u8 = 1;

// the core part has the code that can run on any platform
mod core {
    pub mod memstore;
    pub mod instance;
    pub mod item;
    pub mod id;
    pub mod proto;
    pub mod error;
    pub mod types;
}

pub use core::*;


// platform specific stuff
pub mod platform {
    #[cfg(feature = "wasm-target")]
    pub mod wasm;

    //#[cfg(feature = "os-target")]
    pub mod os;
}



