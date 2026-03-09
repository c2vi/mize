use crate::mize_err;
use crate::MizeResult;
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, RwLock};

use crate::item::ItemData;
use crate::Mize;
use crate::MizeError;

type ConfigThunk = Box<dyn Fn() -> ItemData + Send + Sync>;

pub struct ConfigOpt {
    pub name: String,
    pub val: Option<ItemData>,
    pub thunk: Option<ConfigThunk>,
}

#[derive(Clone)]
pub struct ConfigOptNameAndMize {
    pub name: String,
    pub mize: Mize,
}

impl ConfigOptNameAndMize {
    pub fn default_val(self, val: ItemData) -> ConfigOptNameAndMize {
        let mut config_opts = self.mize.config_opts.lock().unwrap();
        let opt = config_opts.get_mut(&self.name).unwrap();
        opt.val = Some(val);
        self.clone()
    }
    pub fn fun(self, thunk: ConfigThunk) -> Self {
        let mut config_opts = self.mize.config_opts.lock().unwrap();
        let opt = config_opts.get_mut(&self.name).unwrap();
        opt.thunk = Some(thunk);
        self.clone()
    }
}

pub fn gather_config(mize: &mut Mize) -> MizeResult<()> {
    let mut config_opts = mize.config_opts.lock().unwrap();
    let parts = mize.parts.lock().unwrap();

    for part in parts.values() {
        part.as_deref().unwrap().opts(&mut mize.clone());
    }

    // populate values from config files
    let config_file_paths = env::var("MIZE_CONFIG_FILES")?;
    for config_file_path in config_file_paths.split(":") {
        println!("reading config file: {config_file_path}");
        let content = std::fs::read_to_string(config_file_path)?;
        let mut data = ItemData::from_toml(content.as_str())?;
        println!("config data: {data}");
        for path in data.get_paths_recursive()? {
            let conf_name = path.replace("/", ".");
            println!("adding config {conf_name} from config file {config_file_path}");
            let val = data.get_path(path.split("/").collect::<Vec<&str>>())?;
            match config_opts.get_mut(&conf_name) {
                Some(opt) => {
                    opt.val = Some(val);
                }
                None => {
                    config_opts.insert(
                        conf_name.clone(),
                        ConfigOpt {
                            name: conf_name,
                            val: Some(val),
                            thunk: None,
                        },
                    );
                }
            };
        }
    }

    Ok(())
}
