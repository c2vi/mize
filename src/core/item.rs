use log::{trace, debug, info, warn, error};
use serde::Serialize;
use core::fmt;
use std::{i128, string};
use std::{collections::HashMap, path::PathBuf, io::Cursor, fmt::Display};
use std::fs::File;
use tokio::sync::Mutex;
use std::rc::Rc;
use colored::Colorize;
use std::io;

use crate::error::{MizeError, MizeResult, IntoMizeResult};
use crate::instance::Instance;
use crate::instance::store::Store;
use crate::id::MizeId;
use ciborium::Value as CborValue;


// a item always has to do with a Instance, which takes care of how it is updated
#[derive(Debug)]
pub struct Item<'a, S: Store + Sized> {
    id: MizeId,
    pub instance: &'a Instance<S>
}

// without an Instance it is not an item, but only the "data of an item"
// and this type for now is just an alias to CborValue
#[derive(Debug, Clone)]
pub struct ItemData ( pub CborValue );

impl<S: Store> Item<'_, S> {
    pub fn id(&self) -> MizeId {
        self.id.clone()
    }

    pub fn new(id: MizeId, instance: &Instance<S>) -> Item<S> {
        Item { id, instance }
    }

    pub fn value_raw(&self) -> MizeResult<Vec<u8>> {
        // this will call from the instance which gets the value from the store
        self.instance.store.get_value_raw(self.id())
    }
    pub fn as_data_full(&self) -> MizeResult<ItemData> {
        self.instance.store.get_value_data_full(self.id())
    }
    pub fn merge<V: Into<ItemData>>(&mut self, mut value: V) -> MizeResult<()> {
        //trace!("[ {} ] Item.merge()", "CALL".yellow());
        let mut old_data = self.instance.store.get_value_data_full(self.id())?;
        //trace!("old_data: {:?}", old_data);
        old_data.merge(value.into());
        //trace!("new_data: {:?}", old_data);
        self.instance.store.set(self.id(), old_data)?;
        Ok(())
    }
}

impl<S: Store> Display for Item<'_, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let item_data = self.as_data_full().map_err(|_| std::fmt::Error)?;
        write!(f, "{}", item_data);
        return Ok(());
    }
}

impl ItemData {
    pub fn new() -> ItemData {
        ItemData(CborValue::Null)
    }

    pub fn merge(&mut self, other: ItemData) {
        item_data_merge(self, other)
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

fn item_data_get_path(data: &CborValue, path: Vec<String>) -> MizeResult<&CborValue> {
    let mut path_iter = path.clone().into_iter();
    let path_el = match path_iter.nth(0) {
        Some(val) => val,
        None => return Ok(data), // our base case
    };

    let mut sub_data = &CborValue::Null;
    match data {
        CborValue::Map(ref map) => {
            for (key, val) in map {
                if let CborValue::Text(key_str) = key {
                    if key_str == &path_el {
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

fn item_data_set_path(data: &mut CborValue, path: Vec<String>, data_to_set: &CborValue) -> MizeResult<()> {
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

fn item_data_merge(first: &mut ItemData, other: ItemData){
    //trace!("[ {} ] item_data_merge()", "CALL".yellow());
    //trace!("[ {} ] first: {}", "ARG".yellow(), first.clone().into_item_data());
    //trace!("[ {} ] other: {}", "ARG".yellow(), other.clone().into_item_data());

    match first.0 {
        CborValue::Map(ref mut map) => {
            //trace!("data is a map: {:?}", map);
            match other.0 {
                CborValue::Map(mut other_map) => {
                    let mut to_add = Vec::new();
                    for (other_key, other_val) in other_map.clone() {
                        let mut key_found_in_old = false;
                        for (key, mut val) in map.clone() {
                            if key == other_key {
                                val = other_val.clone();
                                key_found_in_old = true;
                            }
                        }
                        if key_found_in_old == false {
                            to_add.push((other_key.clone(), other_val.clone()));
                        }
                    }
                    other_map.extend(to_add);
                },
                _ => {
                    *first = other;
                }
            }
        },
        _ => {
            *first = other;
        },
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
