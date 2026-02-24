#[cfg(feature = "target-os")]
mod cli;
#[cfg(feature = "target-os")]
pub use cli::cli;

mod js;

#[cfg(feature = "target-os")]
pub fn habitica(mize: &mut mize::Mize) {
    use deno_core::{ascii_str_include, include_js_files};

    js::part_from_file(
        mize,
        "habitica",
        ascii_str_include!("./deno_dist/habitica.js"),
    );
}
