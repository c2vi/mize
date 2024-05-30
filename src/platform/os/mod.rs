use ciborium::Value as CborValue;
use serde::Deserialize;
use std::fs;

use crate::id::MizeId;
use crate::instance::store::Store;
use crate::instance::Instance;
use crate::error::{IntoMizeResult, MizeError, MizeResult};


pub fn os_instance_init<S: Store>(instance: Instance<S>) -> MizeResult<()> {
    // this is the code, that runs to initialize an Instance on a system with an os present.

    // load MIZE_CONFIG_FILE as the instance, which is item 0
    let config_file_path = std::env::var("MIZE_CONFIG_FILE")
        .mize_result_msg("Could not get the MIZE_CONFIG_FILE Environment Variable")?;

    let toml_string = fs::read_to_string(&config_file_path)
        .mize_result_msg(format!("Could not read file {} as mize config", &config_file_path))?;

    let toml_deserializer = toml::Deserializer::new(&toml_string.as_str());

    let config = CborValue::deserialize(toml_deserializer)
        .mize_result_msg(format!("Could not deserialize the content of MIZE_CONFIG_FILE at {}", &config_file_path))?;

    instance.set("0", config)?;
    
    return Ok(());
}



