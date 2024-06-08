use core::fmt;
use std::collections::HashMap;
use std::fs::create_dir;
use std::sync::{Arc, Mutex};
use log::{trace, debug, info, warn, error};
use uuid::Uuid;
use std::fs::File;
use colored::Colorize;
use std::path::Path;
use interner::shared::VecStringPool;

use crate::error::{MizeError, MizeResult, IntoMizeResult, MizeResultTrait};
use crate::instance;
use crate::instance::store::Store;
use crate::instance::connection::Connection;
use crate::id::{IntoMizeId, MizeId};
use crate::instance::subscription::Subscription;
use crate::item::{Item, ItemData};
use crate::memstore::MemStore;

pub mod connection;
pub mod store;
pub mod subscription;

static MSG_CHANNEL_SIZE: usize = 200;

/// The Instance type is the heart of the mize system
#[derive(Clone)]
pub struct Instance<S: Store> {
    // question vec of stores???
    // would mean replication logic and such things is handeled by the instance
    // i think it would be better thogh to then implement a ReplicatedStore
    pub store: S,
    peers: Arc<Mutex<Vec<Box<dyn Connection>>>>,
    subs: Arc<Mutex<HashMap<MizeId, Subscription>>>,
    pub id_pool: VecStringPool,
}


/*
pub enum RealmId {
    Uuid(Uuid),
    Local(Vec<String>),
    Tld(Vec<String>),
}
*/

impl Instance<MemStore> {
    pub fn empty() -> Instance<MemStore> {
        let store = MemStore::new();
        let id_pool = VecStringPool::default();
        let peers = Arc::new(Mutex::new(Vec::new()));
        let subs = Arc::new(Mutex::new(HashMap::new()));
        let mut instance = Instance {store, peers, subs, id_pool };
        return instance;
    }
    pub fn new() -> MizeResult<Instance<MemStore>> {
        trace!("[ {} ] Instance::new()", "CALL".yellow());

        let mut instance = Instance::empty();

        instance.init();

        debug!("instance inited with conifg: {}", instance.get("0/config")?.as_data_full()?);

        return Ok(instance);
    }

    fn init(&mut self) -> MizeResult<()> {

        // platform specific init code
        if cfg!(feature = "os-target") { // on os platforms
            crate::platform::os::os_instance_init(self)?
        }

        // end of platform specific init code

        Ok(())
    }

    pub fn with_config(config: ItemData) -> MizeResult<Instance<MemStore>> {
        trace!("[ {} ] Instance::with_config()", "CALL".yellow());
        trace!("config: {}", config);
        let mut instance = Instance::empty();
        instance.set("0", config.clone());
        instance.init()?;

        // set it again, so that the passed config data has presidence over anything the init would set
        debug!("overwriting instance config again with the one passed to Instance::with_config()");
        instance.set("0", config);

        //debug!("instance inited with conifg: {}", instance.get("0/config")?);
        debug!("instance inited with conifg: no");

        Ok(instance)
    }
}

impl<S: Store> Instance<S> {
    pub fn new_item(&self) -> MizeResult<Item<S>> {
        let id = self.id_from_string(self.store.new_id()?);
        return Ok(Item::new(id, &self));
    }

    pub fn get<I: IntoMizeId<S>>(&self, id: I) -> MizeResult<Item<S>> {
        let id = id.to_mize_id(self);
        return Ok(Item::new(id, &self));
    }

    pub fn set<I: IntoMizeId<S>, V: Into<ItemData>>(&mut self, id: I, value: V) -> MizeResult<()> {
        let id = id.to_mize_id(self);
        let item_data = value.into();
        let mut item = self.get(id.clone())?;
        item.merge(item_data)?;
        Ok(())
    }

    pub fn new_id<T: IntoMizeId<S>>(&self, value: T) -> MizeId {
        value.to_mize_id(self)
    }

    pub fn id_from_string(&self, string: String) -> MizeId {
        let vec_string: Vec<String> = string.split("/").map(|v| v.to_owned()).collect();
        return MizeId { path: self.id_pool.get(vec_string) };
    }
    pub fn id_from_vec_string(&self, vec_string: Vec<String>) -> MizeId {
        return MizeId { path: self.id_pool.get(vec_string) };
    }
//impl<T: Into<String>> From<T> for MizeId {
    //fn from(value: T) -> Self {
    //}
//}

}

impl<S: Store> fmt::Debug for Instance<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Mize Instance with subs: {:?}", self.subs,)
    }
}

