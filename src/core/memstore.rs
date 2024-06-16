use std::collections::binary_heap::Iter;
use std::collections::HashMap;
use std::iter::Map;
use std::sync::{Arc, Mutex};
use ciborium::Value as CborValue;
use log::trace;
use colored::Colorize;

use crate::id::MizeId;
use crate::instance::store::{Store, IdIter};
use crate::error::{MizeError, MizeResult, IntoMizeResult};
use crate::item::{Item, ItemData};
use crate::instance::Instance;

#[derive(Clone, Debug)]
pub struct MemStore {
    inner: Arc<Mutex<MemStoreInner>>,
}

pub type MemStoreId = u64;

#[derive(Debug)]
struct MemStoreInner {
    map: HashMap<u64, ItemData>,
    next_id: u64,
}

impl Store for MemStore {
    fn set(&self, id: MizeId, data: ItemData) -> MizeResult<()> {
        let mut inner = self.inner.lock()?;

        inner.map.insert(id_to_u64(id)?, data);
        return Ok(())
    }
    fn get_links(&self, item: Item) -> MizeResult<Vec<MizeId>> {
        let inner = self.inner.lock()?;

        Ok(Vec::new())
    }
    fn get_backlinks(&self, item: Item) -> MizeResult<Vec<MizeId>> {
        let inner = self.inner.lock()?;

        Ok(Vec::new())
    }
    fn new_id(&self) -> MizeResult<String> {
        let mut inner = self.inner.lock()?;

        let id_string = format!("{}", inner.next_id);
        inner.next_id += 1;
        return Ok(id_string);
    }
    fn get_value_raw(&self, id: MizeId) -> MizeResult<Vec<u8>> {
        let inner = self.inner.lock()?;

        let cbor_val = inner.map.get(&id_to_u64(id.clone())?)
            .ok_or(MizeError::new()
            .msg(format!("Item with store_part: {}, id: {} does not exist in MemStore", id_to_u64(id.clone())?, id)))?;

        let path = id.path();
        let mut id_iter = path.iter();
        id_iter.next();
        let id_without_first = id_iter.collect();

        Ok(get_raw_from_cbor(&cbor_val.0, id_without_first)?.to_owned())
    }

    fn get_value_data_full(&self, id: MizeId) -> MizeResult<ItemData> {
        let inner = self.inner.lock()?;

        let data = match inner.map.get(&id_to_u64(id.clone())?) {
            Some(val) => val.to_owned(),
            None => ItemData::new(),
        };

        let tmp = id.path();
        let mut path_iter = tmp.into_iter();
        path_iter.next();
        let new_path: Vec<String> = path_iter.map(|v| v.to_owned()).collect();
        let ret_data = data.get_path(new_path)?;

        return Ok(ret_data);
    }
    fn id_iter(&self) -> MizeResult<IdIter> {

        IdIter::new(Box::new(self.to_owned()))
    }

    fn next_id(&self, prev_id_str: &str) -> MizeResult<Option<String>> {
        println!("next_id: prev_id_str: {}", prev_id_str);
        let inner = self.inner.lock()?;

        let mut id = str_to_u64(prev_id_str)?;

        loop {

            id += 1;

            if id > inner.next_id {
                return Ok(None);
            }

            if inner.map.contains_key(&id) {
                println!("returning: {}", id);
                return Ok(Some(format!("{}", id)));

            } else {
                // try id +1
                continue;
            }
        }
    }

    fn first_id(&self) -> MizeResult<String> {
        Ok("0".to_owned())
    }
}

impl MemStore {
    pub fn new() -> MemStore {
        let inner = MemStoreInner { map: HashMap::new(), next_id: 0 };
        return MemStore { inner: Arc::new(Mutex::new(inner)) };
    }
}

fn id_to_u64(id: MizeId) -> MizeResult<u64> {
    str_to_u64(id.store_part())
}

fn str_to_u64(id_str: &str) -> MizeResult<u64> {
    id_str.parse()
        .mize_result_msg(format!("Could not parse the store_part of mizeid {} into a u64 for the MemStore", id_str))
}

pub fn get_raw_from_cbor<'a>(value: &'a CborValue, path: Vec<&String>) -> MizeResult<&'a [u8]> {
    trace!("[ {} ] get_raw_from_cbor()", "CALL".yellow());
    trace!("[ {} ] value: {:?}", "ARG".yellow(), value);
    trace!("[ {} ] path: {:?}", "ARG".yellow(), path);

    let mut path_iter = path.clone().into_iter();
    let path_el = match path_iter.nth(0) {
        Some(val) => val,
        None => {
            // our base case
            let ret_val = match value {
                CborValue::Bytes(vec) => {
                    trace!("[ {} ] ret value: {:?}", "RET".yellow(), &vec[..]);
                    return Ok(&vec[..]);
                },
                CborValue::Text(string) => { 
                    trace!("[ {} ] ret value: {:?}", "RET".yellow(), string.as_bytes());
                    return Ok(string.as_bytes());
                },
                other => {
                    return Err(MizeError::new()
                        .msg("path is empty and the value is neither Bytes nor Text"));
                },
            };
        }, 
    };
    trace!("hoooooooo: {:?}", path_el);
    
    match value {
        CborValue::Bytes(vec) => {
            trace!("[ {} ] ret value: {:?}", "RET".yellow(), &vec[..]);
            return Ok(&vec[..]);
        },
        CborValue::Text(string) => { 
            trace!("[ {} ] ret value: {:?}", "RET".yellow(), string.as_bytes());
            return Ok(string.as_bytes());
        },
        CborValue::Map(map) => {
            let mut inner_val: &CborValue = &CborValue::Null;
            let mut inner_val_found = false;

            // find the inner_val at path
            for (key, val) in map.into_iter() {
                if let CborValue::Text(key_text) = key {
                    if key_text == path_el {
                        inner_val = val;
                        inner_val_found = true;
                    }
                }
            }
            
            // if there is no value at that path
            if inner_val_found == false {
                return Err(MizeError::new()
                    // jesus christ, this is a pfusch...
                    .msg(format!("get_raw_from_cbor: Path '{}' not Found", path.clone().into_iter().map(|v| v.to_owned()).collect::<Vec<String>>().join("/"))))
            }

            //path_iter.next();
            let inner_path = path_iter.collect();
            trace!("hiiiiii: {:?}", inner_path);
            let ret_value = get_raw_from_cbor(inner_val, inner_path);
            trace!("[ {} ] ret value: {:?}", "RET".yellow(), ret_value);
            return ret_value;
        },
        _ => Err(MizeError::new().msg("get_raw_from_cbor: value is not a map, text or bytes")),
    }
}



