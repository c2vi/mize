use std::collections::HashMap;

use clap::crate_version;
use cli_part::Cli;
use config::Config;
use deno_part::JsPart;
use mize::{
    DynMizePartGuard, Mize, MizeError, MizePart, MizePartGuard, MizeResult, add_parts, mize_err,
};

fn main() {
    let mut mize = Mize::new();

    add_parts!(&mut mize, Cli);

    let cli = mize.get_part_native::<Cli>("cli");

    cli.with_cmd(|cmd| {
        cmd.version(crate_version!())
            .name("ppc")
            .author("ppc")
            .about("the ppc desktop program")
    });

    cli.subcommand(Command::new("test"), |sub_matches| {
        println!("test ppc...");
        Ok(())
    });

    mize.run();
}
