use std::collections::HashMap;

use config::Config;
use deno_part::JsPart;
use mize::{Mize, MizeError, MizePart, MizeResult, add_parts, mize_err};
use clap::crate_version;

fn main() {
    let mut mize = Mize::new();

    add_parts!(&mut mize, Cli::new(&mut mize, "ppc"));
    let cli = Cli::get(&mut mize);
    cli.with_cmd(|cmd| cmd.version(crate_version!()).author("ppc").about("the ppc desktop program"));
    cli.subcommand("test", |sub_matches| {
        println!("test ppc...");
        Ok(())
    });
    
    mize.run();
}
