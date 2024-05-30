use std::path::PathBuf;

use clap::ArgMatches;
use home::home_dir;

use mize::error::{IntoMizeResult, MizeError, MizeResult, MizeResultTrait};
use mize::instance::Instance;



pub fn get(sub_matches: &ArgMatches) -> MizeResult<()> {

    let instance = Instance::new()?;

    let id = sub_matches.get_one::<String>("id")
        .ok_or(MizeError::new().msg("No id Argument specified"))?;

    let item = instance.get(id)?;

    println!("{}", item);

    return Ok(());
}

