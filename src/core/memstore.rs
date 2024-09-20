use std::collections::binary_heap::Iter;
use std::collections::HashMap;
use std::iter::Map;
use std::sync::{Arc, Mutex};
use ciborium::Value as CborValue;
use tracing::{instrument, trace};
use colored::Colorize;

use crate::id::MizeId;
use crate::instance::store::{Store, IdIter};
use crate::error::{MizeError, MizeResult, IntoMizeResult};
use crate::item::{Item, ItemData};
use crate::instance::Instance;
use crate::item::get_raw_from_cbor;
use crate::mize_err;

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

        if let Some(old_data) = inner.map.get(&id_to_u64(id.clone())?) {
            //if there is already data there, set at the correct path
            let mut old_data = old_data.to_owned();
            let path = id.after_store_part();
            old_data.set_path(path, data)?;
            inner.map.insert(id_to_u64(id)?, old_data);

        } else {
            // if no data exists for that store_part, just insert
            inner.map.insert(id_to_u64(id)?, data);
        }

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

    #[instrument(name="fn.MemStore::get_value_raw" skip(self))]
    fn get_value_raw(&self, id: MizeId) -> MizeResult<Vec<u8>> {
    trace!("[ {} ] id: {:?}", "ARG".yellow(), id);
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

        let new_path = id.after_store_part();
        let ret_data = data.get_path(new_path)?;



    #[cfg(feature = "wasm-target")]
    unsafe {
// console_log macro
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}
macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}
//end of console_log macro
        console_log!("ret_data in data full in memstore: {:?}", ret_data);
    }




        return Ok(ret_data);
    }
    fn id_iter(&self) -> MizeResult<IdIter> {

        IdIter::new(Box::new(self.to_owned()))
    }

    fn next_id(&self, prev_id_str: &str) -> MizeResult<Option<String>> {
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




