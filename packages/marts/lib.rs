use mize::MizeResult;

#[cfg(feature = "target-os")]
mod cli;
#[cfg(feature = "target-os")]
pub use cli::*;

#[cfg(feature = "target-os")]
pub mod habitica;
#[cfg(feature = "target-os")]
pub use habitica::*;

pub mod js;
pub use js::*;
