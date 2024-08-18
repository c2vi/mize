use std::path::PathBuf;

use clap::ArgMatches;
use home::home_dir;

use mize::error::{IntoMizeResult, MizeError, MizeResult, MizeResultTrait};
use mize::instance::Instance;
use mize::platform::os::config_from_cli_args;

use mize::platform::os::fsstore::FileStore;
//use self::FileStore;


pub fn is_running(sub_matches: &ArgMatches) -> MizeResult<()> {

    let home_dir = env!("HOME");

    if FileStore::store_is_opened(home_dir.to_owned() + "/.mize")? {
        println!("true");
    } else {
        println!("false");
    }

    Ok(())
 
}

