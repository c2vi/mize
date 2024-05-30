use std::collections::HashMap;
use std::iter::Map;
use std::sync::{Arc, Mutex};
use ciborium::Value as CborValue;
use log::trace;

use crate::id::MizeId;
use crate::instance::store::Store;
use crate::error::{MizeError, MizeResult, IntoMizeResult};
use crate::item::{Item, ItemData};

#[derive(Clone, Debug)]
pub struct MemStore {
    inner: Arc<Mutex<MemStoreInner>>,
}

#[derive(Debug)]
struct MemStoreInner {
    map: HashMap<u64, CborValue>,
    next_id: u64,
}

impl Store for MemStore {
    fn set<T: Into<ItemData>>(mut self, id: &MizeId, data: T) -> MizeResult<()> {
        let mut inner = self.inner.lock()?;


        inner.map.insert(id_to_u64(id)?, data.into());
        return Ok(())
    }
    fn get_links(self, item: Item<Self>) -> MizeResult<Vec<MizeId>> {
        let inner = self.inner.lock()?;

        Ok(Vec::new())
    }
    fn get_backlinks(self, item: Item<Self>) -> MizeResult<Vec<MizeId>> {
        let inner = self.inner.lock()?;

        Ok(Vec::new())
    }
    fn new_id(&self) -> MizeResult<MizeId> {
        let inner = self.inner.lock()?;

        let id_string = format!("{}", inner.next_id);
        return Ok(id_string.into());
    }
    fn get_value_raw(&self, id: &MizeId) -> MizeResult<Vec<u8>> {
        let inner = self.inner.lock()?;

        let cbor_val = inner.map.get(&id_to_u64(id)?)
            .ok_or(MizeError::new()
            .msg(format!("Item with store_part: {}, id: {} does not exist in MemStore", id_to_u64(id)?, id)))?;

        let mut id_without_first = id.clone().as_vec();
        id_without_first.remove(0);
        get_raw_from_cbor(cbor_val.to_owned(), id_without_first)
    }
}

impl MemStore {
    pub fn new() -> MemStore {
        let inner = MemStoreInner { map: HashMap::new(), next_id: 0 };
        return MemStore { inner: Arc::new(Mutex::new(inner)) };
    }
}

fn id_to_u64(id: &MizeId) -> MizeResult<u64> {
    id.store_part()
        .parse()
        .mize_result_msg(format!("Could not parse the store_part of mizeid {} into a u64 for the MemStore", id))
}

fn get_raw_from_cbor(value: CborValue, mut path: Vec<String>) -> MizeResult<Vec<u8>> {
    match value {
        CborValue::Bytes(vec) => Ok(vec),
        CborValue::Text(string) => Ok(string.into_bytes()),
        CborValue::Map(map) => {
            let mut inner_val: CborValue = CborValue::Null;
                trace!("map: {:?}", map);
            for (key, val) in map.into_iter() {
                let first_path_string = path.first()
                    .ok_or(MizeError::new().msg("get_raw_from_cbor: path is empty"))?;

                if let CborValue::Text(key_text) = key {
                    if key_text == *first_path_string {
                        inner_val = val
                    }
                }
            }
            if inner_val == CborValue::Null {
                return Err(MizeError::new().msg(format!("get_raw_from_cbor: Path {} not Found", path.join("/"))))
            }
            path.remove(0);
            get_raw_from_cbor(inner_val, path)
        },
        _ => Err(MizeError::new().msg("get_raw_from_cbor: value is not Bytes, Text or Map")),
    }
}



