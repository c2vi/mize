use std::path::PathBuf;

use clap::ArgMatches;
use home::home_dir;

use mize::error::{IntoMizeResult, MizeError, MizeResult, MizeResultTrait};
use mize::instance::Instance;
use mize::platform::os::config_from_cli_args;



pub fn set(sub_matches: &ArgMatches) -> MizeResult<()> {

    let instance = Instance::with_config(config_from_cli_args(sub_matches)?)?;

    let id = sub_matches.get_one::<String>("id")
        .ok_or(MizeError::new().msg("No id Argument specified"))?;

    let item = instance.get(id)?;

    let recurse_count = sub_matches.get_count("recurse");

    println!("{}", item.as_data_full()?);
    return Ok(());

    if recurse_count > 0 {
        println!("{}", item.id());
        println!("{}", item.as_data_full()?);
    } else {
        println!("{}", item);
    };

    return Ok(());
}
