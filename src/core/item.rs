use log::{trace, debug, info, warn, error};
use std::{collections::HashMap, path::PathBuf, io::Cursor, fmt::Display};
use std::fs::File;
use tokio::sync::Mutex;
use std::rc::Rc;

use crate::error::{MizeError, MizeResult, IntoMizeResult};
use crate::instance::Instance;
use crate::instance::store::Store;
use crate::id::MizeId;
use ciborium::Value as CborValue;


// a item always has to do with a Instance, which takes care of how it is updated
#[derive(Debug)]
pub struct Item<'a, S: Store + Sized> {
    pub id: MizeId,
    pub instance: &'a Instance<S>
}

// without an Instance it is not an item, but only the "data of an item"
// and this type for now is just an alias to CborValue
pub type ItemData = CborValue;

impl<S: Store> Item<'_, S> {
    pub fn id(&self) -> MizeId {
        self.id.clone()
    }

    pub fn value_raw(&self) -> MizeResult<Vec<u8>> {
        // this will call from the instance which gets the value from the store
        self.instance.store.get_value_raw(self.id())
    }
}

impl<S: Store> Display for Item<'_, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.value_raw() {
            Ok(value) => match String::from_utf8(value.to_owned()) {
                Ok(string) => write!(f, "{}", string),
                Err(_) => write!(f, "{:?}", self.value_raw()),
            }
            Err(err) => {err.log(); Ok(())},
        };
        return Ok(());
    }
}




/*
#[derive(Debug)]
pub enum Item {
    Cbor(ItemCbor),
    Folder(ItemFolder),
    Ref(ItemRef),
}

#[derive(Clone, Debug)]
pub struct ItemCbor {
    pub inner: CborValue,
    pub id: MizeId,
}

#[derive(Debug)]
pub struct ItemFolder {
    pub path: PathBuf,
    pub id: MizeId,
}

#[derive(Debug)]
pub struct ItemRef {
    pub store: Mutex<Rc<Itemstore>>,
    pub id: MizeId,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct MizeId {
    pub main: u64,
    pub path: Option<Vec<String>>
}

pub struct MizeType {
}


//////////////////////////////////////////////////////////////////////////////////////////////


impl Display for MizeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(path) = &self.path {
            write!(f, "{}/{}", self.main, path.join("/"))
        } else {
            write!(f, "{}", self.main)
        }
    }
}

impl MizeId {
    pub fn main(main: u64) -> MizeId {
        MizeId { main, path: None }
    }
    pub fn join(self, segment: &str) -> MizeId {
        let mut new = self.clone();
        match new.path {
            None => {
                new.path = Some(vec![segment.to_owned()]);
            }
            Some(ref mut val) => {
                val.push(segment.to_owned());
            }
        }
        return new;
    }
}



//////////////////////////////////////////////////////////////////////////////////////////////

impl ItemCbor {
    fn value_as_file(&self) -> MizeResult<File> {
        /* sadly does not work like that
        let cbor_c = self.inner.get("v").ok_or(MizeError::new()
            .msg(format!("The ItemCbor ({}) does not have a \"c\" field ... so can't get it as a file", self.id()?)))?;
        match cbor_c {
            Cbor::Bytes(val) => {
                return Ok(Cursor::new(val.clone()).into());
            },
            _ => {
                return Err(MizeError::new().msg(format!("The value fo item ({}) is not of type Cbor::Bytes", self.id()?)));
            },
        }
        */
        return File::open(self.value_as_path()?).mize_result_msg("Could not open file");
    }

    fn value_as_path(&self) -> MizeResult<PathBuf> {
        todo!()
    }

    /* an old way of getting the id this
    fn id(self) -> MizeResult<MizeId> {
        let cbor_item = self.inner.get("item".to_owned()).ok_or(MizeError::new()
            .msg("ItemCbor had no path: \"item\""))?;

        let cbor_id = match cbor_item {
            Cbor::Map(map) => {
                map.get("id").ok_or(MizeError::new().msg("ItemCbor had no path \"item/id\""))
            },
            _ => {
                return Err(MizeError::new().msg("ItemCbor path \"item\" was not of type Cbor::Map"));
            },
        };

        let id = match cbor_id {
            Cbor::Unsigned(val) => val.into_u64(),
            _ => {
                return Err(MizeError::new().msg("ItemCbor path \"item/id\" was not of type Cbor::Unsigned"));
            },
            
        };

        return Ok(MizeId { inner: id });
    }
    // */

    fn id(&self) -> MizeResult<MizeId> {
        Ok(self.id.clone())
    }
    fn as_cbor(&self) -> MizeResult<ItemCbor> {
        Ok(self.clone())
    }
    fn write_to_path(&self, path: PathBuf) -> MizeResult<()> {
        todo!()
        //let cbor_content = 
        //std::fs::write(path.join(format!("{}", self.id)), );
    }
}

impl ItemFolder {
    fn value_as_file(&self) -> MizeResult<File> {
        let file = File::open(self.path.join("v"))
            .mize_result_msg(format!("Could not open the value file of item ({})", self.id()?));
        return file;
    }

    fn value_as_path(&self) -> MizeResult<PathBuf> {
        return Ok(self.path.clone());
    }

    fn id(&self) -> MizeResult<MizeId> {
        Ok(self.id.clone())
    }

    fn as_cbor(&self) -> MizeResult<ItemCbor> {
        todo!()
    }

    fn write_to_path(&self, path: PathBuf) -> MizeResult<()> {
        Ok(())
    }
}

impl ItemRef {
    fn value_as_file(&self) -> MizeResult<File> {
        todo!()
    }

    fn value_as_path(&self) -> MizeResult<PathBuf> {
        todo!()
    }

    fn id(&self) -> MizeResult<MizeId> {
       return Ok(self.id.clone());
    }

    fn as_cbor(&self) -> MizeResult<ItemCbor> {
        todo!()
    }

    async fn write_to_path(&self, path: PathBuf) -> MizeResult<()> {
        let store = self.store.lock().await;
        for (key, value) in self.map.iter() {
            // TODO right here!!!!
        }
            todo!()
    }
}

impl Item {
    pub fn value_as_file(&self) -> MizeResult<File> {
        match self {
            Item::Cbor(val) => val.value_as_file(),
            Item::Ref(val) => val.value_as_file(),
            Item::Folder(val) => val.value_as_file(),
        }
    }

    pub fn value_as_path(&self) -> MizeResult<PathBuf> {
        match self {
            Item::Cbor(val) => val.value_as_path(),
            Item::Ref(val) => val.value_as_path(),
            Item::Folder(val) => val.value_as_path(),
        }
    }

    pub fn id(&self) -> MizeResult<MizeId> {
        match self {
            Item::Cbor(val) => val.id(),
            Item::Ref(val) => val.id(),
            Item::Folder(val) => val.id(),
        }
    }

    pub fn as_cbor(&self) -> MizeResult<ItemCbor> {
        match self {
            Item::Cbor(val) => val.as_cbor(),
            Item::Ref(val) => val.as_cbor(),
            Item::Folder(val) => val.as_cbor(),
        }
    }

    pub fn write_to_path(&self, path: PathBuf) -> MizeResult<()> {
        match self {
            Item::Cbor(val) => val.write_to_path(path),
            Item::Ref(val) => val.write_to_path(path),
            Item::Folder(val) => val.write_to_path(path),
        }
    }
    pub fn load_json(json_str: &str) -> MizeResult<Item> {
        todo!()
    }
}
*/

