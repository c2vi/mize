
#![ allow( warnings ) ]

//#![no_std]
//extern crate std;

mod core {
    pub mod mme;
    pub mod slot;
    pub mod presenter;
    pub mod layout;
    pub mod adapter;
}

pub use core::slot;
pub use core::presenter;
pub use core::layout;
pub use core::adapter;
pub use core::mme;


pub mod implementors {
    pub mod html;

    #[cfg(feature = "qt")]
    pub mod qt_widget;

    #[cfg(feature = "slint")]
    pub mod slint_widget;

    #[cfg(feature = "x11")]
    pub mod x_window;
}
