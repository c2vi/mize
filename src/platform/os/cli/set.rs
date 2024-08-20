use std::path::PathBuf;

use clap::ArgMatches;
use home::home_dir;

use mize::error::{IntoMizeResult, MizeError, MizeResult, MizeResultTrait};
use mize::instance::Instance;
use mize::item::IntoItemData;
use mize::platform::os::config_from_cli_args;



pub fn set(sub_matches: &ArgMatches) -> MizeResult<()> {

    let instance = Instance::with_config(config_from_cli_args(sub_matches)?)?;

    let id = sub_matches.get_one::<String>("id")
        .ok_or(MizeError::new().msg("No id Argument specified"))?;

    let value = sub_matches.get_one::<String>("value")
        .ok_or(MizeError::new().msg("No value Argument specified"))?;

    instance.set(id, value.into_item_data())?;

    instance.wait_for_updaater_thread()?;

    Ok(())
}
