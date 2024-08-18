use std::path::PathBuf;

use clap::ArgMatches;
use home::home_dir;

use mize::error::{IntoMizeResult, MizeError, MizeResult, MizeResultTrait};
use mize::instance::Instance;
use mize::platform::os::config_from_cli_args;



pub fn create(sub_matches: &ArgMatches) -> MizeResult<()> {

    let mut instance = Instance::with_config(config_from_cli_args(sub_matches)?)?;

    let item = instance.new_item()?;

    println!("id: {}", item.id());

    return Ok(());
}

