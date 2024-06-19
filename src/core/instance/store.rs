use std::iter::Map;
use std::option::Iter;
use dyn_clone::DynClone;

use crate::error::MizeResult;
use crate::item::{Item, ItemData};
use crate::id::MizeId;
use crate::memstore::MemStore;
use crate::instance::Instance;
use crate::item::get_raw_from_cbor;

dyn_clone::clone_trait_object!(Store);

pub trait Store: DynClone + Send + Sync {
    // should have the ability to be multi threaded, if the underlying implementation supports 
    // multithreaded IO operations
    // for now all refs to a store hold a Mutex but this mutex is part of where the MizeStore trait
    // is implemented, so that there can be a multithreaded implementation
    //fn get(self, id: MizeId) -> MizeResult<Item<Self>> where Self: Sized;

    fn set(&self, id: MizeId, data: ItemData) -> MizeResult<()>;

    // in the future should implement transactions, ....

    // funcs to do with links, backlinks
    fn get_links(&self, item: Item) -> MizeResult<Vec<MizeId>>;

    fn get_backlinks(&self, item: Item) -> MizeResult<Vec<MizeId>>;

    fn new_id(&self) -> MizeResult<String>;

    fn get_value_raw(&self, id: MizeId) -> MizeResult<Vec<u8>>;

    fn get_value_data_full(&self, id: MizeId) -> MizeResult<ItemData>;

    fn id_iter(&self) -> MizeResult<IdIter>;

    fn next_id(&self, prev_id: &str) -> MizeResult<Option<String>>;

    fn first_id(&self) -> MizeResult<String>;
}

pub struct IdIter {
    cur_id: String,
    store: Box<dyn Store>,
}

impl IdIter {
    pub fn new(store: Box<dyn Store>) -> MizeResult<IdIter> {
        Ok(IdIter { cur_id: store.first_id()?, store })
    }
}

impl Iterator for IdIter {
    type Item = MizeResult<String>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.store.next_id(&self.cur_id) {
            Err(err) => {
                Some(Err(err))
            },
            Ok(Some(new_id)) => {
                self.cur_id = new_id.clone();
                Some(Ok(new_id))
            },
            Ok(None) => {
                None
            },
        }
    }
}

