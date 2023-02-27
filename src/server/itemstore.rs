
use std::net::ToSocketAddrs;
use std::{io, fmt::format, vec};
use surrealdb::{Datastore, Key, Val};
use itertools::Itertools;
use crate::error;
use crate::error::MizeError;
use crate::server::proto;
use crate::server::Mutexes;

use serde_json::Value as JsonValue;
use serde::{Serialize, Deserialize};

use super::proto::Delta;

// The struct to do the storage and updates to it
// is responsible, that no illegal states can occur in the storage, by using transactions
// currently it's just a surrealdb Datastore. should be replaced by a completely custom File-System in the future.
pub struct Itemstore {
    db: surrealdb::Datastore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    #[serde(flatten)]
    pub json: JsonValue,
}

impl Item {
    pub fn get_id(&self) -> Result<String, MizeError> {
        if let JsonValue::Object(ob) = &self.json{
            if let Some(id_val) = ob.get("mize-id") {
                if let JsonValue::String(id) = id_val {
                    return Ok(id.to_string())
                } else {
                    return Err(MizeError::new(11).extra_msg("mize-id field in an item was not a String"));
                }
            } else {
                return Err(MizeError::new(11).extra_msg("There was no mize-id field in an item"));
            }
        } else {
            return Err(MizeError::new(11).extra_msg("An Item was not a JsonValue::Object"));
        }
    }

    pub fn apply_delta(&mut self, delta: Delta) -> Result<(), MizeError> {
        for (path, value) in delta.raw {
            let mut to_replace = &mut self.json;
            for name in path {
                to_replace = if to_replace.get(&name).is_none() {
                    return Err(MizeError::new(11)
                        .extra_msg("Error applying a Delta to an Item. The path could not be followed."));
                } else {
                    to_replace.get_mut(&name).ok_or(MizeError::new(11))?
                };
            }
            *to_replace = value;
        }

        return Ok(())
    }

    pub fn from_bytes(vec: Vec<u8>) -> Result<Item, MizeError> {
        let string = String::from_utf8(vec)?;
        let item = serde_json::from_str(&string)?;
        return Ok(item);
    }
}

impl Itemstore {
    pub async fn new(path: String) -> Result<Itemstore, MizeError> {
        let ds = Datastore::new(&(String::from("file://") + &path[..] + "/db")[..]).await
            .map_err(|_| MizeError::new(30).format(vec![&path]))?;

        let mut tr = ds.transaction(true, false).await?;

        match tr.get(0u64.to_le_bytes()).await {
            Ok(Some(val)) => println!("item 0 already created"),
            Ok(None) => {
                let data = serde_json::json!({
                        "num_of_items": 1,
                        "next_free_id": 1,
                        "_commit": 0,
                        "mize-type": "mize-main",
                        "mize-id": "0"
                });

                tr.set(0u64.to_be_bytes(), data.to_string()).await?;
                tr.commit().await?;
            },

            Err(e) => {return Err(MizeError::from(e))},
        };
        return Ok(Itemstore {db:ds});
    }

    pub async fn create(&self, mut item: Item, mutexes: Mutexes) -> Result<u64, MizeError> {
        let mut tr = self.db.transaction(true, false).await?;

        //get next_free_id
        let json: JsonValue = serde_json::from_str(&String::from_utf8(tr.get(0u64.to_be_bytes()).await?.ok_or(MizeError::new(31))?)
            .map_err(|_| MizeError::new(11))?)
            .map_err(|_| MizeError::new(11))?;

        let new_id: u64 = json.get("next_free_id").ok_or(MizeError::new(11))?.as_u64().ok_or(MizeError::new(11))?;

        let num_of_items: u64 = json.get("num_of_items").ok_or(MizeError::new(11))?.as_u64().ok_or(MizeError::new(11))?;

        let new_next_id = new_id +1;
        let new_num = num_of_items + 1;

        //add mize-id field to item
        let delta: Delta = serde_json::from_str(&format!("[[[\"mize-id\"], \"{}\"]]", new_id))?;
        item.apply_delta(delta)?;

        tr.set(new_id.to_be_bytes(), item.json.to_string()).await?;

        tr.commit().await?;

        //handle update to item0
        let delta_item0: Delta = serde_json::from_str(&format!("[[[\"num_of_items\"], {}], [[\"next_free_id\"], {}]]", new_num, new_next_id))?;
        proto::handle_update(proto::MizeId::Local(0), delta_item0, mutexes.clone(), None).await?;

        return Ok(new_id);
    }


    pub async fn update(&self, id: u64, delta: Delta) -> Result<(), MizeError>{

        let mut tr = self.db.transaction(true, true).await?;

        let item_bytes: Vec<u8> = tr.get(id.to_le_bytes()).await?
            .ok_or(MizeError::new(11)
                .extra_msg("The Item an update should be applied to does not exist."))?;

        let mut item = Item::from_bytes(item_bytes)?;

        item.apply_delta(delta)?;

        tr.set(id.to_be_bytes(), serde_json::to_string(&item)?).await?;

        tr.commit().await?;
        return Ok(());
    }



    pub async fn delete(&self, id: u64) -> Result<(), error::MizeError>{
        let mut tr = self.db.transaction(true, true).await?;

        tr.del(id.to_be_bytes()).await?;

        tr.commit().await?;

        return Ok(());
    }


    pub async fn get(&self, id: u64) -> Result<Item, MizeError> {
        let mut tr = self.db.transaction(false, false).await?;

        let item_res = tr.get(id.to_le_bytes()).await?;
        if let Some(item_vec) = item_res {
            let item_str: String = String::from_utf8(item_vec).map_err(|_|MizeError::new(11))?;
            let item: JsonValue = serde_json::from_str(&item_str).map_err(|_|MizeError::new(11))?;
            return Ok(Item { json: item })
        } else {
            return Err(MizeError::new(32).format(vec![&format!("{}", id)]));
        }
        return Err(MizeError::new(11));
    }
}

