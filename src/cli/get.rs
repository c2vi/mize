use std::path::PathBuf;

use clap::ArgMatches;
use home::home_dir;

use crate::error::{MizeResultTrait, IntoMizeResult, MizeError};
use crate::instance::Instance;



pub async fn get(sub_matches: &ArgMatches) {
    let instance_folder_path = match sub_matches.get_one::<String>("store") {
        Some(a) => PathBuf::from(a),
        None => {
            let mut home_dir = home_dir()
            .ok_or(MizeError::new().category("io").category("env").msg("could not get the home directory, where the mize store is by default")).critical();
            home_dir.push(".mize");
            home_dir
        },
    };
    let instance = Instance::new(instance_folder_path);
}

