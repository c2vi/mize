use mize::MizeResult;

#[cfg(feature = "target-os")]
mod cli;
#[cfg(feature = "target-os")]
pub use cli::*;

#[cfg(feature = "target-os")]
pub mod habitica;
#[cfg(feature = "target-os")]
pub use habitica::*;

#[cfg(feature = "target-os")]
pub mod c2vi;
#[cfg(feature = "target-os")]
pub use c2vi::*;

pub mod js;
pub use js::*;
