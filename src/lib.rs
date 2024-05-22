
#![ allow( warnings ) ]

static PROTO_VERSION: u8 = 1;

// the core part houses the code that can run on any platform
/*
pub mod core {
    pub mod memstore;
    pub mod instance;
    pub mod item;
    pub mod id;
    pub mod proto;
    pub mod error;
    pub mod types;
}

pub use core::*;
// */

pub mod platform {
    #[cfg(feature = "wasm-target")]
    pub mod wasm;
}



