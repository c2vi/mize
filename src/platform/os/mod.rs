use ciborium::Value as CborValue;
use serde::Deserialize;
use std::env::VarError;
use std::fs;
use tracing::{debug, error, info, trace, warn};
use clap::ArgMatches;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use sha2::{Sha256, Sha512, Digest};
use flate2::bufread::GzDecoder;
use tar::Archive;
use std::fs::File;
use std::io::copy;

use crate::id::MizeId;
use crate::instance::store::Store;
use crate::instance::Instance;
use crate::error::{IntoMizeResult, MizeError, MizeResult};
use crate::item::{data_from_string, IntoItemData, ItemData};
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
            trace!("config after MIZE_CONFIG_FILE env var: {}", instance.get("self/config")?);
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
            let config = data_from_string(config_string)?;
            debug!("env var MIZE_CONFIG present");
            instance.set_blocking("self/config", config)?;
            trace!("config after MIZE_CONFIG env var: {}", instance.get("self/config")?);
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
    let mut store_path = match instance.get("self/config/store_path")?.value_string() {
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

            instance.set_blocking("self/config/store_path", store_path.clone().into_item_data())?;

            let store_path = instance.get("self/config/store_path")?.value_string()?;

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
            error!("CONNECTING... would connect, but that is not imon't identify as anything, to not cause anyone problems... So that everybody can be satisfied...plemented on windows yet");
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

pub fn get_module_hash(instance: &mut Instance, name: &str, mut options: ItemData) -> MizeResult<(String, String)> {

    let mut selector_data = instance.get("self/config/selector")?.as_data_full()?;

    // handle the case, when we want a module for not the system we are running on...
    println!("get_module_hash: name: {}", name);
    let mut name_parts = name.split(".");
    if name_parts.next().ok_or(mize_err!("no 0th element in the modName"))? == "cross" {
        let system = name_parts.next().ok_or(mize_err!("no 1th element in the modName"))?;
        let mod_name: String = name_parts.collect();
        println!("get_module_hash: mod_name: {}", mod_name);
        if mod_name == "" {
            return Err(mize_err!("no 2nd element in the modName"));
        }

        selector_data.set_path("system", system)?;
        selector_data.set_path("modName", mod_name)?;
    } else {
        selector_data.set_path("modName", name)?;
    }

    selector_data.merge(options);

    // important to produce the same hash for the sama selector data
    selector_data.sort_keys();

    let selector_str = selector_data.to_json()?;

    let mut hasher = Sha256::new();

    hasher.update(selector_str.clone().as_bytes());

    let result = hasher.finalize();

    let mut hex: String = result.iter()
        .map(|b| format!("{:02x}", b).to_string())
        .collect::<Vec<String>>()
        .join("");

    hex.truncate(32);

    return Ok((hex, selector_str));

}

fn download_module(instance: &mut Instance, store_path: &str, module_url: &str, module_hash: &str, module_name: &str, module_selector: &str) -> MizeResult<()> {


    let tmp_file_path_gz = format!("{}/{}.tar.gz", store_path, module_hash);

     let tmp_gz_file = if !PathBuf::from(tmp_file_path_gz.clone()).exists() {

        let mut tmp_gz_file = File::create(&tmp_file_path_gz)?;

        debug!("downloading module '{}' with hash: {}", module_name, module_hash);

        let response = http_req::request::get(format!("http://{}/mize/dist/{}-{}.tar.gz", module_url, module_hash, module_name), &mut tmp_gz_file)?;

        let code = response.status_code();
        if code != 200.into() {
            return Err(mize_err!("failed to get module '{}' with hash '{}' and selector: {}", module_name, module_hash, module_selector).msg(format!("the http request to fetch the module returned a status code of: {}", code)));
        }

        tmp_gz_file
    } else {
        std::fs::File::open(tmp_file_path_gz.clone())?
    };

    let tar = GzDecoder::new(std::io::BufReader::new(tmp_gz_file));
    let mut archive = Archive::new(tar);

    //archive.set_mask(umask::Mode::parse("rwxrwxrwx")?.into());

    let target_dir = format!("{}/modules/{}", store_path, module_hash);

    std::fs::create_dir_all(target_dir.as_str())?;

    archive.unpack(target_dir.as_str())?;

    fs::remove_file(&tmp_file_path_gz)?;

    Ok(())
}

pub fn fetch_module(instance: &mut Instance, module_name: &str) -> MizeResult<String> {

    let module_name_with_slashes = module_name.replace(".", "/");
    if let Ok(module_path) = instance.get(format!("self/config/module_dir/{}", module_name_with_slashes))?.value_string() {
        // in case a dir is configured, which holds the modules output
        return Ok(module_path);
    } else {
        // if null, load it from the url
        let module_url = instance.get("self/config/module_url")?.value_string()?;

        let store_path = instance.get("self/config/store_path")?.value_string()?;

        let (module_hash, module_selector) = get_module_hash(instance, module_name, ItemData::new())?;

        if !PathBuf::from(format!("{}/modules/{}", store_path, module_hash.as_str())).exists() {
            // download module if it does not exist
            download_module(instance, store_path.as_str(), module_url.as_str(), module_hash.as_str(), module_name, module_selector.as_str())?;
        }

        return Ok(format!("{}/modules/{}", store_path, module_hash.as_str()));
    }
}

pub fn load_module(instance: &mut Instance, module_name: &str, path: Option<String>) -> MizeResult<()> {
    
    let module_path = if path.is_some() { 
        format!("{}/lib/libmize_module_{}.so", path.unwrap(), module_name)

    } else { 
        format!("{}/lib/libmize_module_{}.so", fetch_module(instance, module_name)?, module_name) 
    };

    let lib = unsafe { libloading::Library::new(module_path)? };

    let func: libloading::Symbol<unsafe extern "C" fn(&mut Box<dyn Module + Send + Sync>, Instance) -> ()> = unsafe { lib.get(format!("get_mize_module_{}", module_name).as_bytes())? };

    let mut module: Box<dyn Module + Send + Sync> = Box::new(EmptyModule {});

    unsafe { func(&mut module, instance.clone()) };

    let mut modules_inner = instance.modules.lock()?;

    module.init(&instance)?;

    modules_inner.insert(module_name.to_owned(), module);

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
        config.merge(data_from_string(config_string.to_string())?);
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




