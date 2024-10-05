use ciborium::Value as CborValue;
use serde::Deserialize;
use std::env::VarError;
use std::fs;
use tracing::{debug, error, info, trace, warn};
use clap::ArgMatches;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use sha2::{Sha256, Sha512, Digest};
use flate2::read::GzDecoder;
use tar::Archive;
use std::fs::File;
use std::io::copy;

use crate::id::MizeId;
use crate::instance::store::Store;
use crate::instance::Instance;
use crate::error::{IntoMizeResult, MizeError, MizeResult};
use crate::item::{ItemData, IntoItemData};
use crate::memstore::MemStore;
use crate::{mize_err, Module};
use crate::instance::module::EmptyModule;

use self::fsstore::FileStore;

pub mod fsstore;

#[cfg(target_family = "unix")]
mod unix_socket;

#[cfg(target_family = "unix")]
use crate::platform::os::unix_socket::UnixListener;
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
        Ok(path) => {
            path
        },
        Err(e) => {
            // the default store_path: $HOME/.mize
            let home_dir = env!("HOME");
            if home_dir == "" {
                return Err(mize_err!("env var $HOME empty"));
            }

            let store_path = home_dir.to_owned() + "/.mize";

            instance.set_blocking("0/config/store_path", store_path.clone().into_item_data())?;

            let store_path = instance.get("0/config/store_path")?.value_string()?;

            println!("store_path: {}", store_path);

            store_path
        },
    };

    if FileStore::store_is_opened(store_path.to_owned())? {
        // if the store is already opened, connect to the instance, that opened it and join
        // it's namespace

        #[cfg(target_family = "unix")]
        {
            info!("CONNECTING");
            use crate::platform::os::unix_socket;
            unix_socket::connect(instance, store_path.into())?;
            return Ok(());
        }

        #[cfg(target_family = "windows")]
        {
            warn!("CONNECTING... would connect, but that is not implemented on windows yet");
        }

    } else {
        // else open it ourselves
        let file_store = FileStore::new(store_path.as_str())?;
        instance.migrate_to_store(Box::new(file_store))?;

        let path = Path::new(&store_path).to_owned();

        #[cfg(target_family = "unix")]
        {
            instance.add_listener(UnixListener::new(path)?)?;
        }

        #[cfg(target_family = "windows")]
        {
            warn!("would add a Listener on a local socket, but that is not yet implemented for windows");;
        }
    }

    Ok(())
}

pub fn seconds_since_modification(path: &Path) -> MizeResult<u64> {
    let metadata = fs::metadata(path)?;

    let modified = metadata.modified()?;

    let duration = modified.duration_since(UNIX_EPOCH)?;

    Ok(duration.as_secs())
}

pub fn get_module_hash(instance: &mut Instance, name: &str, mut options: ItemData) -> MizeResult<String> {

    let mut selector_data = instance.get("0/config/selector")?.as_data_full()?;

    selector_data.set_path("name", name)?;

    selector_data.merge(options);

    let selector_str = selector_data.to_json()?;

    let mut hasher = Sha256::new();

    hasher.update(selector_str.as_bytes());

    let result = hasher.finalize();

    let mut hex: String = result.iter()
        .map(|b| format!("{:02x}", b).to_string())
        .collect::<Vec<String>>()
        .join("");

    hex.truncate(32);

    return Ok(hex);

}

fn fetch_module(instance: &mut Instance, store_path: &str, module_url: &str, module_hash: &str, module_name: &str) -> MizeResult<()> {

    let tmp_file_path_gz = format!("{}/{}.tar.gz", store_path, module_hash);
    let mut tmp_gz_file = File::create(&tmp_file_path_gz)?;

    http_req::request::get(format!("http://{}/mize/dist/{}-{}.tar.gz", module_url, module_hash, module_name), &mut tmp_gz_file)?;

    let tar = GzDecoder::new(tmp_gz_file);
    let mut archive = Archive::new(tar);

    //archive.set_mask(umask::Mode::parse("rwxrwxrwx")?.into());

    let target_dir = format!("{}/modules/{}", store_path, module_hash);

    std::fs::create_dir_all(target_dir.as_str())?;

    archive.unpack(target_dir.as_str())?;

    fs::remove_file(&tmp_file_path_gz)?;

    Ok(())
}

pub fn load_module(instance: &mut Instance, module_name: &str, path: Option<PathBuf>) -> MizeResult<()> {

    let module_dir_str = instance.get(format!("0/config/module_dir/{}", module_name))?.value_string();

    let module_path = match module_dir_str {
        Err(_) => {
            // if null, load it from the url
            let module_url = instance.get("0/config/module_url")?.value_string()?;

            let store_path = instance.get("0/config/store_path")?.value_string()?;

            let module_hash = get_module_hash(instance, name, ItemData::new())?;

            if !PathBuf::from(format!("{}/modules/{}", store_path, module_hash.as_str())).exists() {
                // fetch module if it does not exist
                fetch_module(instance, store_path.as_str(), module_url.as_str(), module_hash.as_str(), module_name.as_str())?;
            }

            // the module_path
            format!("{}/modules/{}/lib/libmize_module_{}.so", store_path, module_hash, module_name)
        },
        Ok(module_dir_str) => {
            // the module_path with the module_dir from 0/config
            format!("{}/lib/libmize_module_{}.so", module_dir_str.as_str(), module_name)
        }
    };

    let lib = unsafe { libloading::Library::new(module_path)? };

    let func: libloading::Symbol<unsafe extern "C" fn(&mut Box<dyn Module + Send + Sync>) -> ()> = unsafe { lib.get(format!("get_mize_module_{}", name).as_bytes())? };

    let mut module: Box<dyn Module + Send + Sync> = Box::new(EmptyModule {});

    unsafe { func(&mut module) };

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



