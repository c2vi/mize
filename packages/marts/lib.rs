use mize::MizeResult;

#[cfg(feature = "target-os")]
mod cli;
#[cfg(feature = "target-os")]
pub use cli::*;

pub mod js;
pub use js::*;

#[cfg(feature = "target-os")]
pub fn habitica(mize: &mut mize::Mize) -> MizeResult<()> {
    use deno_core::ascii_str_include;

    js::part_from_file(
        mize,
        "habitica",
        ascii_str_include!("./deno_dist/habitica.js"),
    )?;

    Ok(())
}
