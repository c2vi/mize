use ciborium::Value as CborValue;
use serde::Deserialize;
use std::env::VarError;
use std::fs;
use tracing::{debug, error, info, trace, warn};
use clap::ArgMatches;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use crate::id::MizeId;
use crate::instance::store::Store;
use crate::instance::Instance;
use crate::error::{IntoMizeResult, MizeError, MizeResult};
use crate::item::{ItemData, IntoItemData};
use crate::memstore::MemStore;
use crate::{mize_err, Module};
use crate::platform::os::unix_socket::UnixListener;
use crate::instance::module::EmptyModule;

use self::fsstore::FileStore;

pub mod fsstore;
mod unix_socket;
//mod web;


pub fn os_instance_init(instance: &mut Instance) -> MizeResult<()> {
    // this is the code, that runs to initialize an Instance on a system with an os present.

    ////// load MIZE_CONFIG_FILE as the instance, which is item 0
    match std::env::var("MIZE_CONFIG_FILE") {
        Ok(config_file_path) => {
            let config = config_from_file(config_file_path)?;
            debug!("env var MIZE_CONFIG_FILE present");
            instance.set_blocking("0", config)?;
            trace!("config after MIZE_CONFIG_FILE env var: {}", instance.get("0/config")?);
        }
        Err(var_err) => match var_err {
            VarError::NotPresent => {
                debug!("env var MIZE_CONFIG_FILE NOT present");
            },
            VarError::NotUnicode(_) => {
                warn!("env var MIZE_CONFIG_FILE is not Unicode, so not reading it")
            },
        }
    };

    ////// load config from MIZE_CONFIG env var
    match std::env::var("MIZE_CONFIG") {
        Ok(config_string) => {
            let config = config_from_string(config_string)?;
            debug!("env var MIZE_CONFIG present");
            instance.set_blocking("0", config)?;
            trace!("config after MIZE_CONFIG env var: {}", instance.get("0/config")?);
        }
        Err(var_err) => match var_err {
            VarError::NotPresent => {
                debug!("env var MIZE_CONFIG NOT present")
            },
            VarError::NotUnicode(_) => {
                warn!("env var MIZE_CONFIG is not Unicode, so not reading it")
            },
        }
    };

    ////// if a config.store_path is set, upgrade to the filestore there
    let mut test = instance.get("0")?.as_data_full()?;
    let mut store_path = match instance.get("0/config/store_path")?.value_string() {
        Ok(path) => path,
        Err(e) => {
            // the default store_path: $HOME/.mize
            let home_dir = env!("HOME");
            if home_dir == "" {
                return Err(mize_err!("env var $HOME empty"));
            }
            home_dir.to_owned() + "/.mize"
        },
    };

    if FileStore::store_is_opened(store_path.to_owned())? {
        // if the store is already opened, connect to the instance, that opened it and join
        // it's namespace
        info!("CONNECTING");
        unix_socket::connect(instance, store_path.into())?;
        return Ok(());

    } else {
        // else open it ourselves
        let file_store = FileStore::new(store_path.as_str())?;
        instance.migrate_to_store(Box::new(file_store))?;

        let path = Path::new(&store_path).to_owned();
        instance.add_listener(UnixListener::new(path)?)?;
    }

    Ok(())
}

pub fn seconds_since_modification(path: &Path) -> MizeResult<u64> {
    let metadata = fs::metadata(path)?;

    let modified = metadata.modified()?;

    let duration = modified.duration_since(UNIX_EPOCH)?;

    Ok(duration.as_secs())
}

fn get_correct_module_path(module_dir: &Path, name: &str) -> MizeResult<PathBuf> {
    let mut release_path = module_dir.join("modules").join(name).join("target").join("release").join(format!("libmize_module_{}.so", name));
    let mut debug_path = module_dir.join("modules").join(name).join("target").join("debug").join(format!("libmize_module_{}.so", name));

    if !release_path.exists() && !debug_path.exists() {
        release_path = module_dir.join("modules").join(name).join("target").join("release").join(format!("lib{}.so", name));
        debug_path = module_dir.join("modules").join(name).join("target").join("debug").join(format!("lib{}.so", name));
    }

    let release_modtime = match seconds_since_modification(&release_path) {
        Ok(secs) => secs,
        Err(e) => { return Ok(debug_path); },
    };

    let debug_modtime = match seconds_since_modification(&debug_path) {
        Ok(secs) => secs,
        Err(e) => { return Ok(release_path); },
    };

    if seconds_since_modification(&release_path)? > seconds_since_modification(&debug_path)? {
        return Ok(debug_path);
    } else {
        return Ok(release_path);
    };

}

pub fn load_module(instance: &mut Instance, name: &str, path: Option<PathBuf>) -> MizeResult<()> {

    let module_dir_str = instance.get("0/config/module_dir")?.value_string()?;
    let module_dir = Path::new(&module_dir_str);

    let module_path = get_correct_module_path(module_dir, name)?;

    let lib = unsafe { libloading::Library::new(module_path)? };

    let func: libloading::Symbol<unsafe extern "C" fn(&mut Box<dyn Module + Send + Sync>) -> ()> = unsafe { lib.get(format!("get_mize_module_{}", name).as_bytes())? };

    let mut module: Box<dyn Module + Send + Sync> = Box::new(EmptyModule {});

    unsafe { func(&mut module) };
    println!("hiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiii load module");


    let mut modules_inner = instance.modules.lock()?;

    module.init(&instance);

    modules_inner.insert(name.to_owned(), module);

    unsafe {
        // dropping the lib, would (i suspect) free all memory, of the library's code, which would
        // make the module's vtable point into empty memory
        std::mem::forget(lib);
    }

    Ok(())
}

pub fn config_from_cli_args(matches: &ArgMatches) -> MizeResult<ItemData> {

    let mut config = ItemData::new();

    if let Some(config_file_path) = matches.get_one::<String>("config-file") {
        config.merge(config_from_file(config_file_path.to_string())?);
        trace!("config after --config-file arg: {}", config);
    }

    if let Some(config_string) = matches.get_one::<String>("config") {
        config.merge(config_from_string(config_string.to_string())?);
        trace!("config after --config arg: {}", config);
    }

    return Ok(config);
}


pub fn config_from_file(file_path: String) -> MizeResult<ItemData> {

    let toml_string = fs::read_to_string(&file_path)
        .mize_result_msg(format!("Could not read file {} as mize config", &file_path))?;

    let item_data = ItemData::from_toml(toml_string.as_str())?;

    return Ok(item_data);
}

fn config_from_string(config_string: String) -> MizeResult<ItemData> {

    let mut config = ItemData::new();

    for option in config_string.split(";") {
        let path = option.split("=").nth(0)
            .ok_or(MizeError::new().msg(format!("Failed to parse Option: option '{}' has an empty path (thing beforee =)", option)))?;
        let value = option.split("=").nth(1)
            .ok_or(MizeError::new().msg(format!("Failed to parse Option: option '{}' has an empty value (thing after =)", option)))?;
        let mut path_vec = vec!["config"];
        path_vec.extend(path.split("."));

        config.set_path(path_vec, ItemData::parse(value))?;
    }

    return Ok(config);
}



