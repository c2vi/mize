use crate::MizeResult;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::item::ItemData;
use crate::Mize;

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

    Ok(())
}
