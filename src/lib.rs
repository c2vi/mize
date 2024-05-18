
#![ allow( warnings ) ]

static PROTO_VERSION: u8 = 1;

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




