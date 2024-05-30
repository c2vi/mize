use core::fmt;
use std::collections::HashMap;
use std::fs::create_dir;
use std::sync::{Arc, Mutex};
use log::{trace, debug, info, warn, error};
use uuid::Uuid;
use std::fs::File;
use colored::Colorize;
use std::path::Path;

use crate::error::{MizeError, MizeResult, IntoMizeResult, MizeResultTrait};
use crate::instance::store::Store;
use crate::instance::connection::Connection;
use crate::id::MizeId;
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
}


/*
pub enum RealmId {
    Uuid(Uuid),
    Local(Vec<String>),
    Tld(Vec<String>),
}
*/

impl Instance<MemStore> {
    pub fn new() -> MizeResult<Instance<MemStore>> {
        trace!("[ {} ] Instance::new()", "CALL".yellow());

        let memstore = MemStore::new();
        let instance = Instance {store: memstore, peers: Arc::new(Mutex::new(Vec::new())), subs: Arc::new(Mutex::new(HashMap::new())) };


        // platform specific init code
        if cfg!(feature = "os-target") { // on os platforms
            crate::platform::os::os_instance_init(instance.clone())?
        }

        // end of platform specific init code


        return Ok(instance);
    }
}

impl<S: Store> Instance<S> {
    pub fn new_item(self) -> MizeResult<Item<S>> {
        let id = self.store.new_id()?;
        return Ok(Item {id, instance: self});
    }

    pub fn get<I: Into<MizeId>>(self, id: I) -> MizeResult<Item<S>> {
        return Ok(Item {id: id.into(), instance: self});
    }

    pub fn set<I: Into<MizeId>, V: Into<ItemData>>(self, id: I, value: V) -> MizeResult<()> {
        self.store.set(&id.into(), value)
    }

}

impl<S: Store> fmt::Debug for Instance<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Mize Instance with subs: {:?}", self.subs,)
    }
}

