use tracing::{debug, error, info, instrument, trace, warn};
use serde::Serialize;
use core::fmt;
use std::{i128, string};
use std::{collections::HashMap, path::PathBuf, io::Cursor, fmt::Display};
use std::fs::File;
use tokio::sync::Mutex;
use std::rc::Rc;
use colored::Colorize;
use std::io;
use tracing::{span, Level};

use crate::error::{MizeError, MizeResult, IntoMizeResult};
use crate::instance::Instance;
use crate::instance::store::Store;
use crate::id::MizeId;
use ciborium::Value as CborValue;


// a item always has to do with a Instance, which takes care of how it is updated
#[derive(Debug)]
pub struct Item<'a> {
    id: MizeId,
    pub instance: &'a Instance
}

// without an Instance it is not an item, but only the "data of an item"
// and this type for now is just an alias to CborValue
#[derive(Debug, Clone)]
pub struct ItemData ( pub CborValue );

impl Item<'_> {
    pub fn id(&self) -> MizeId {
        self.id.clone()
    }

    pub fn new(id: MizeId, instance: &Instance) -> Item {
        Item { id, instance }
    }

    pub fn value_raw(&self) -> MizeResult<Vec<u8>> {
        // this will call from the instance which gets the value from the store
        self.instance.store.get_value_raw(self.id())
    }

    pub fn value_string(&self) -> MizeResult<String> {
        let raw = self.value_raw()?;
        String::from_utf8(raw.clone())
            .mize_result_msg(format!("failed to convert raw value from item '{}' into a utf-8 string. raw value was: '{:02X?}'", self.id(), raw))
    }

    pub fn as_data_full(&self) -> MizeResult<ItemData> {
        self.instance.store.get_value_data_full(self.id())
    }

    pub fn merge<V: Into<ItemData>>(&mut self, mut value: V) -> MizeResult<()> {
        let mut old_data = self.instance.store.get_value_data_full(self.id())?;

        let new_data = value.into();

        old_data.merge(new_data);

        self.instance.store.set(self.id(), old_data)?;
        Ok(())
    }
}

impl<'a> Display for Item<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //let item_data = self.as_data_full().map_err(|_| std::fmt::Error)?;
        write!(f, "Item with id: {}", self.id());
        return Ok(());
    }
}

impl ItemData {
    pub fn new() -> ItemData {
        ItemData(CborValue::Null)
    }

    pub fn merge(&mut self, other: ItemData) {
        item_data_merge(&mut self.0, &other.0);
    }

    pub fn cbor(&self) -> &CborValue {
        &self.0
    }

    pub fn null() -> CborValue {
        CborValue::Null
    }

    pub fn parse<T: Into<String>>(value: T) -> ItemData {
        let value_str = value.into();
        if value_str == "false" {
            return CborValue::Bool(false).into_item_data();
        }
        if value_str == "false" {
            return CborValue::Bool(true).into_item_data();
        }
        if let Ok(int) = value_str.parse::<i128>() {
            return int.into_item_data()
        }
        return CborValue::Text(value_str).into_item_data();
    }

    pub fn set_path<P: IntoPath, D: IntoItemData>(&mut self, path: P, value: D) -> MizeResult<()> {
        //trace!("[ {} ] ItemData::set_path()", "CALL".yellow());
        let path = path.into_path();
        let value = value.into_item_data();

        item_data_set_path(&mut self.0, path, &value.0)
    }

    pub fn get_path<P: IntoPath>(&self, path: P) -> MizeResult<ItemData> {
        let path = path.into_path();
        Ok(item_data_get_path(&self.0, path)?.to_owned().into_item_data())
    }

}

pub fn item_data_get_path(data: &CborValue, path: Vec<String>) -> MizeResult<&CborValue> {
    let mut path_iter = path.clone().into_iter();
    //println!("data: {:?}", data);
    //println!("path: {:?}", path);

    let path_el = match path_iter.nth(0) {
        Some(val) => val,
        None => return Ok(data), // our base case
    };

    let mut sub_data = &CborValue::Null;
    match data {
        CborValue::Map(ref map) => {
            for (key, val) in map {
                //println!("key: {:?}", key);
                //println!("path_el: {}", path_el);
                if let CborValue::Text(key_str) = key {
                    if key_str == &path_el {
                        println!("setting sub_data to: {:?}", val);
                        sub_data = val;
                    }
                }
            }
        },
        val => {
            return Err(MizeError::new()
                .msg(format!("Failed to get path '{:?}' from ItemData, the data at '{}' is not a map", path, path_el))
                .msg(format!("{:?} is: {:?}", path_el, val)));
        },
    };
    item_data_get_path(sub_data, path_iter.collect())
}

pub fn item_data_set_path(data: &mut CborValue, path: Vec<String>, data_to_set: &CborValue) -> MizeResult<()> {
    //trace!("[ {} ] item_data_set_path()", "CALL".yellow());
    //trace!("[ {} ] data: {}", "ARG".yellow(), data.clone().into_item_data());
    //trace!("[ {} ] path: {:?}", "ARG".yellow(), path);
    //trace!("[ {} ] data_to_set: {}", "ARG".yellow(), data_to_set.clone().into_item_data());

    let mut path_iter = path.clone().into_iter();
    let path_el = match path_iter.nth(0) {
        Some(val) => val,
        None => {
            // our base case
            *data = data_to_set.to_owned();
            return Ok(());
        }, 
    };
    match data {
        CborValue::Map(ref mut map) => {
            for (key, val) in map {
                if let CborValue::Text(key_str) = key {
                    if key_str == &path_el {
                        path_iter.next();
                        return item_data_set_path(val, path_iter.collect(), data_to_set);
                    }
                }
            }
        },
        CborValue::Null => {
            let mut inner_value = CborValue::Null;
            item_data_set_path(&mut inner_value, path_iter.collect(), data_to_set);

            let map_vec = vec![(CborValue::Text(path_el), inner_value)];
            *data = CborValue::Map(map_vec);

            return Ok(());
        },
        val => {
            return Err(MizeError::new()
                .msg(format!("Failed to get path '{:?}' from ItemData, the data at '{}' is not a map", path, path_el))
                .msg(format!("{:?} is: {:?}", path_el, val)));
        },
    };
    return Err(MizeError::new().msg("unreachable"));
}

pub fn item_data_merge(merge_into: &mut CborValue, other: &CborValue){
    // needs to be recursive

    match (merge_into, other) {
        // if both are maps
        // merge the keys/vals in the maps and call recursively on them
        (CborValue::Map(ref mut merge_into_map), CborValue::Map(ref other_map)) => {
            // go through other_map and merge those keys/values to merge_into
            for (other_key, other_val) in other_map {

                // so find that same vale in merge_into, if exists recursively merge, else just add to
                // vec
                let mut found_value = CborValue::Null;
                let mut value_found = false;
                let mut new_value_for_merge_into: Vec<(CborValue, CborValue)> = Vec::new(); // here we collect the
                                                                           // new contents of the
                                                                           // map, to assign later
                                                                           // to merge_into_map

                for (merge_into_key, merge_into_val) in merge_into_map.clone() {
                    if merge_into_key == *other_key {
                        value_found = true;
                        found_value = merge_into_val.to_owned();
                        // if we found it, also don't add it to new_value_for_merge_into

                    } else {
                        //if not found push to new_value_for_merge_into
                        new_value_for_merge_into.push((merge_into_key.to_owned(), merge_into_val.to_owned()));
                    }
                }

                if value_found {
                    // we found it so merge and asign
                    item_data_merge(&mut found_value, other_val);

                    // to asign push to new_value_for_merge_into, because the old was removed already
                    new_value_for_merge_into.push((other_key.to_owned(), found_value));

                } else {
                    // if not found, just add the other_key/val to new_value_for_merge_into
                    new_value_for_merge_into.push((other_key.to_owned(), other_val.to_owned()));
                }

                // finally, we need to asign the new_value_for_merge_into into merge_into
                *merge_into_map = new_value_for_merge_into;
            }
        }

        // if other is Null, don't asign
        // this is gonna have repercussions.... because it's not clean behaviour
        (merge_into, CborValue::Null) => {
        }

        // in any other case we just want to set merge_into to other
        // also this is the base case
        (merge_into, other) => {
            *merge_into = other.to_owned();
        }
    }
}

#[instrument]
pub fn get_raw_from_cbor<'a>(value: &'a CborValue, path: Vec<&String>) -> MizeResult<&'a [u8]> {
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
            let ret_value = get_raw_from_cbor(inner_val, inner_path);
            trace!("[ {} ] ret value: {:?}", "RET".yellow(), ret_value);
            return ret_value;
        },
        _ => Err(MizeError::new()
            .msg(format!("get_raw_from_cbor: value is not a map, text or bytes ... value: {:?}", value))),
    }
}

trait IntoPath {
    fn into_path(self) -> Vec<String>;
}

pub trait IntoItemData {
    fn into_item_data(self) -> ItemData;
}

impl IntoPath for &str {
    fn into_path(self) -> Vec<String> {
        self.to_owned().split("/").map(|v| v.to_owned()).collect()
    }
}

impl IntoPath for &[&str] {
    fn into_path(self) -> Vec<String> {
        self.into_iter().map(|v| (*v).to_owned()).collect()
    }
}
impl IntoPath for Vec<String> {
    fn into_path(self) -> Vec<String> {
        self
    }
}
impl IntoPath for Vec<&str> {
    fn into_path(self) -> Vec<String> {
        self.into_iter().map(|v| v.to_owned()).collect()
    }
}

impl IntoItemData for &str {
    fn into_item_data(self) -> ItemData {
        ItemData::parse(self)
    }
}
impl IntoItemData for CborValue {
    fn into_item_data(self) -> ItemData {
        ItemData ( self )
    }
}
impl IntoItemData for ItemData {
    fn into_item_data(self) -> ItemData {
        self
    }
}
impl IntoItemData for i128 {
    fn into_item_data(self) -> ItemData {
        ItemData(self.into())
    }
}

impl fmt::Display for ItemData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\nItemData: ");
        let display_writer = DisplayWriter (f);
        self.0.serialize(&mut serde_json::Serializer::pretty(display_writer))
            .map_err(|serde_err| std::fmt::Error)
    }
}

// thanks to: https://stackoverflow.com/a/61768916
struct DisplayWriter<'a, 'b>(&'a mut fmt::Formatter<'b>);

impl<'a, 'b> io::Write for DisplayWriter<'a, 'b> {
    fn write(&mut self, bytes: &[u8]) -> std::result::Result<usize, std::io::Error> {
        
        self.0.write_str(&String::from_utf8_lossy(bytes))
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;

        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::result::Result<(), std::io::Error> { todo!() }
}

