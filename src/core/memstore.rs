use std::collections::HashMap;
use cbor::Cbor as CborValue;

use crate::id::MizeId;
use crate::instance::store::Store;
use crate::error::MizeResult;
use crate::item::{Item, ItemData};

pub struct MemStore {
    map: HashMap<MizeId, CborValue>,
    next_id: u64,
}

impl Store for MemStore {
    fn set<T: Into<ItemData>>(mut self, id: MizeId, data: T) -> MizeResult<()> {
        self.map.insert(id, data.into());
        return Ok(())
    }
    fn get_links(self, item: Item<Self>) -> MizeResult<Vec<MizeId>> {
        Ok(Vec::new())
    }
    fn get_backlinks(self, item: Item<Self>) -> MizeResult<Vec<MizeId>> {
        Ok(Vec::new())
    }
    fn new_id(&self) -> MizeResult<MizeId> {
        let id_string = format!("{}", self.next_id);
        return Ok(id_string.into());
    }
}

impl MemStore {
    pub fn new() -> MemStore {
        return MemStore { map: HashMap::new(), next_id: 0 };
    }
}
