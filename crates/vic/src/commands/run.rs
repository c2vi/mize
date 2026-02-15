use clap::ArgMatches;

use crate::victor::Victor;
use crate::error::VicResult;

pub fn main(matches: &ArgMatches) -> VicResult<()> {

    println!("hiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiii");

    let mut victor = Victor::new()?;

    let item_to_run = match matches.get_one::<String>("runnable") {
        Some(val) => val,
        None => "default",
    };

    let sub_args = matches.get_many::<String>("runnable").unwrap_or_default().map(|v| v.as_str()).collect::<Vec<_>>();

    // check if the item to run is a nix flake url
    if item_to_run.contains("#") {
        victor.run_flake_url(item_to_run)?;
    }

    // check if we have something like vic:rpi
    if item_to_run.contains(":") {
        victor.run_from_resource(item_to_run)?;
    }

    // if we have no '#' or ':' look for the thing in the vicPkgs
    victor.run_from_vic_pkgs(item_to_run, sub_args)?;

    Ok(())
}

