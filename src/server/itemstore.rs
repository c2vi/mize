
use std::net::ToSocketAddrs;
use std::{io, fmt::format, vec};
use surrealdb::kvs::{Datastore, Key, Val};
use itertools::Itertools;
use crate::error;
use crate::error::MizeError;
use crate::server::proto::{self};
use crate::server::Mutexes;
use crate::server::Peer;

use serde_json::Value as JsonValue;
use serde::{Serialize, Deserialize};

use super::proto::Delta;

// The struct to do the storage and updates to it
// is responsible, that no illegal states can occur in the storage, by using transactions
// currently it's just a surrealdb Datastore. should be replaced by a completely custom File-System in the future.
pub struct Itemstore {
    db: Datastore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    #[serde(flatten)]
    pub json: JsonValue,
}

impl Item {
    pub fn get_id(&self) -> Result<String, MizeError> {
        if let JsonValue::Object(ob) = &self.json{
            if let Some(id_val) = ob.get("__item__") {
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
        //println!("DELTA: {:?}", delta);
        for (path, value) in delta.delta {
            let mut to_replace = &mut self.json;
            let mut counter = path.len() as i32;
            for name in path {
                counter -= 1;

//                if let Some(inner) = to_replace.get(&name) {
//                    to_replace = inner

//                } else if let JsonValue::Object(ob) = to_replace {
                    //let mut new_map = JsonValue::Object(serde_json::Map::new());
                    //ob.insert(name, new_map);
                    //to_replace = &mut new_map;
//                } else {
//                    return Err(MizeError::new(11)
//                        .extra_msg("Error applying a Delta to an Item. The path could not be followed."));
//                }


                //on last iteration
                if counter == 0 {
                    if value.is_none() {
                        if to_replace.is_object() {
                            to_replace.as_object_mut().ok_or(MizeError::new(11))?.remove(&name);
                            return Ok(())
                        }
                    }
                }

                to_replace = if to_replace.get_mut(&name).is_some() {
                    to_replace.get_mut(&name).ok_or(MizeError::new(11))?

                } else if to_replace.is_object() {
                // when path does not exist

                    let mut new_map = JsonValue::Object(serde_json::Map::new());
                    to_replace.as_object_mut().ok_or(MizeError::new(11))?.insert(name.clone(), new_map);
                    to_replace.get_mut(&name).ok_or(MizeError::new(11))?

                } else {

                    return Err(MizeError::new(11)
                        .extra_msg("Error applying a Delta to an Item. The path could not be followed."));
                };

            }

            // replace with value from the delta
            *to_replace = value.into();
        }

        println!("After applying a Delta, {}", self.json);
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
        //println!("PATH of DB: {}", path);
        let ds = Datastore::new(&(String::from("file://") + &path[..])[..]).await
            .map_err(|e| MizeError::new(30).extra_msg(format!("surrealdb Error: {}", e)).extra_msg(format!("Trying to create at Location: {}", path)))?;

        let mut tr = ds.transaction(true, false).await?;

        match tr.get(0u64.to_le_bytes()).await {
            Ok(Some(val)) => println!("item 0 already created"),
            Ok(None) => {
                let data = serde_json::json!({
                        "num_of_items": 1,
                        "next_free_id": 1,
                        "__commit__": 0,
                        "__type__": "mize-main",
                        "__item__": "0"
                });

                tr.set(0u64.to_be_bytes(), data.to_string()).await?;
                tr.commit().await?;
            },

            Err(e) => {return Err(MizeError::from(e))},
        };
        return Ok(Itemstore {db:ds});
    }

    pub async fn create(&self, mut item: Item, mutexes: Mutexes, origin: Peer) -> Result<u64, MizeError> {
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
        //let delta: Delta = serde_json::from_str(&format!("[[[\"__item__\"], \"{}\"]]", new_id))?;
        let mut delta = Delta::new();
        delta.append(vec!["__item__"], serde_json::json!(new_id));
        item.apply_delta(delta)?;

        tr.set(new_id.to_be_bytes(), item.json.to_string()).await?;

        tr.commit().await?;

        //handle update to item0
        let mut delta_item0: Delta = Delta::new();
        delta_item0.append(vec!["num_of_items"], serde_json::json!(new_num));
        delta_item0.append(vec!["next_free_id"], serde_json::json!(new_next_id));

        proto::handle_update(vec![(proto::MizeId::new(format!("0")), delta_item0)], origin, mutexes.clone()).await?;

        return Ok(new_id);
    }


    pub async fn update(&self, id: u64, delta: Delta) -> Result<(), MizeError>{
        println!("in itemstore.update");

        let mut tr = self.db.transaction(true, true).await?;

        let item_bytes: Vec<u8> = tr.get(id.to_le_bytes()).await?
            .ok_or(MizeError::new(11)
                .extra_msg("The Item an update should be applied to does not exist."))?;

        let mut item = Item::from_bytes(item_bytes)?;

        item.apply_delta(delta)?;
        //println!("after applying delta: {:?}", item);

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

