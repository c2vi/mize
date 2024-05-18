use std::collections::HashMap;
use std::fs::create_dir;
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
pub struct Instance<S: Store> {
    // question vec of stores???
    // would mean replication logic and such things is handeled by the instance
    // i think it would be better thogh to then implement a ReplicatedStore
    store: S,
    peers: Vec<Box<dyn Connection>>,
    subs: HashMap<MizeId, Subscription>
}

/*
pub enum RealmId {
    Uuid(Uuid),
    Local(Vec<String>),
    Tld(Vec<String>),
}
*/


impl<S: Store> Instance<S> {
    // new() opens a
    pub async fn new() -> MizeResult<Instance<MemStore>> {
        trace!("[ {} ] Instance::new()", "CALL".yellow());

        let memstore = MemStore::new();
        let instance = Instance {store: memstore, peers: Vec::new(), subs: HashMap::new()};

        // platform specific init code
        // TODO

        return Ok(instance);
    }

    pub fn new_item(self) -> MizeResult<Item<S>> {
        let id = self.store.new_id()?;
        return Ok(Item {id, instance: self});
    }

    pub fn get<I: Into<MizeId>>(self, id: I) -> MizeResult<Item<S>> {
        return Ok(Item {id: id.into(), instance: self});
    }

    pub fn set<I: Into<MizeId>, V: Into<ItemData>>(self, id: I, value: V) -> MizeResult<()> {
        self.store.set(id.into(), value)
    }

}


