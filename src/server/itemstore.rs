
use std::net::ToSocketAddrs;
use std::{io, fmt::format, vec};
use surrealdb::{Datastore, Key, Val};
use itertools::Itertools;
use crate::error;
use crate::error::MizeError;
use crate::server::proto;

use super::proto::Delta;

// The struct to do the storage and updates to it
// is responsible, that no illegal states can occur in the storage, by using transactions
// currently it's just a surrealdb Datastore. should be replaced by a completely custom File-System in the future.
pub struct Itemstore {
    db: surrealdb::Datastore,
}

pub type Item = Vec<[Vec<u8>; 2]>;

impl Itemstore {
    pub async fn new(path: String) -> Result<Itemstore, MizeError> {
        let ds = Datastore::new(&(String::from("file://") + &path[..] + "/db")[..]).await.expect("Could not open database");

        let mut tr = ds.transaction(true, false).await?;

        match tr.get(vec!('0' as u8)).await {
            Ok(Some(val)) => println!("item 0 already created"),
            Ok(None) => {
                let keys = vec![
                    "num_of_items".as_bytes().to_vec(),
                    "next_free_id".as_bytes().to_vec(),
                    "_commit".as_bytes().to_vec(),
                    "_type".as_bytes().to_vec(),
                    //"renders".as_bytes().to_vec(),
                    //"modules".as_bytes().to_vec(),
                ];

                tr.set("0".to_string().into_bytes(), encode(keys)).await?;
                let one: u64 = 1;
                let zero: u64 = 0;

                tr.set("0:num_of_items".to_string().into_bytes(), one.to_be_bytes()).await?;
                tr.set("0:next_free_id".to_string().into_bytes(), one.to_be_bytes()).await?;
                tr.set("0:_commit".to_string().into_bytes(), zero.to_be_bytes()).await?;
                tr.set("0:_type".to_string().into_bytes(), "mize-main".as_bytes().to_vec()).await?;
                //tr.set("0:renders".to_string().into_bytes(), Vec::new());
                //tr.set("0:modules".to_string().into_bytes(), Vec::new());

                tr.commit().await?;
            },

            Err(e) => {return Err(MizeError::from(e))},
        };
        return Ok(Itemstore {db:ds});
    }

    pub async fn create(&self, item: Item) -> Result<u64, MizeError> {
        let mut new_id:u64 = 0;

        let mut tr = self.db.transaction(true, false).await?;
        let id = tr.get("0:next_free_id").await
            .expect("no item with id 0 when there definetly should be one")
            .expect("error reading item 0 from db");
        let id_u64 = u64::from_be_bytes(id.clone().try_into().unwrap());
        let mut keys: Vec<Vec<u8>> = Vec::new();

        //set id:_commit
        let key = format!("{}:_commit", id_u64).into_bytes();
        tr.set(key.clone(), vec![0,0,0,0,0,0,0,0]).await?;
        keys.push("_commit".to_owned().into_bytes());


        for field in item {
            let mut key = format!("{}:", id_u64).into_bytes();
            key.extend(field[0].clone());
            keys.push(field[0].clone());

            let val = field[1].clone();
            tr.set(key, val).await?;
        }
        tr.set(format!("{}", id_u64), encode(keys)).await?;

        tr.commit().await?;
        return Ok(new_id)
    }

    pub async fn update(&self, update: proto::Update) -> Result<(), MizeError>{
        println!("itemstore.update func");

        let mut tr = self.db.transaction(true, true).await?;

        //read keys
        let mut old_keys = match tr.get(format!("{}", update.id).into_bytes()).await? {
            Some(keys) => decode(keys),
            None => {
                return Err(MizeError::new(104)
                    .extra_msg("Internal Datastorage Error: the Index of the Item was not found"));
            },
        };

        //write new values
        for (mut key, delta) in update.raw {

            let mut store_key = (update.id.clone() + ":").into_bytes();
            store_key.extend(key.clone());

            let mut new_val = Vec::new();

            if let Some(old_val) = tr.get(store_key.clone()).await? {
                new_val = proto::apply_delta(old_val, delta)?;
                tr.set(store_key.clone(), new_val.clone()).await?;
            } else {
                old_keys.push(key.clone());
                new_val = proto::apply_delta(Vec::new(), delta)?;
                tr.set(store_key.clone(), new_val.clone()).await?;
            }

            //remove field if new_val is empty
            if (new_val.len() == 0) {
                if let Some(key_index) = old_keys.iter().position(|x| *x == key.clone()){
                    old_keys.remove(key_index);
                }
                let mut store_key = update.id.clone().into_bytes();
                store_key.push(':' as u8);
                store_key.extend(key);
                tr.del(store_key).await?;
            } else {
                tr.set(store_key.clone(), new_val).await?;
            }
        }

        //write keys again
        tr.set(update.id.into_bytes(), encode(old_keys)).await?;

        tr.commit().await?;
        return Ok(());
    }



    pub async fn delete(&self, id_str: String) -> Result<(), error::MizeError>{
        let id_u64: u64 = match id_str.parse() {
            Ok(id) => id,
            Err(err) => {
                return Err(MizeError::new(110)
                    .extra_msg("std::num::ParseIntError while parsing id \"{}\" into an Integer"));
            }
        };
        let mut tr = self.db.transaction(true, true).await?;

        let keys_res = tr.get(format!("{}", id_u64).into_bytes()).await?;
        if let Some(keys) = keys_res {
            for key in decode(keys) {
                tr.del(key).await?;
            }
        } else {
            return Err(MizeError::new(104)
                .extra_msg("Internal Datastorage Error: the Index of the Item was not found, \
                    This means most likely that this item does not exist."));
        }

        tr.del(id_str.into_bytes()).await?;

        tr.commit().await?;
        return Ok(());
    }



    pub async fn get(&self, id: u64) -> Result<Item, MizeError> {
        let mut tr = self.db.transaction(false, false).await?;
        let mut item: Item = Vec::new();

        let keys_res = tr.get(format!("{}", id).into_bytes()).await?;
        if let Some(keys) = keys_res {
            for key in decode(keys) {
                let mut field: [Vec<u8>; 2] = [Vec::new(), Vec::new()];
                field[0] = key.clone();
                let mut ke = format!("{}:", id).into_bytes();
                ke.extend(key.clone());
                field[1] = match tr.get(ke).await {
                    Ok(Some(val)) => val,
                    Ok(None) => panic!("a field that should be there wasn't"),
                    Err(e) => panic!("Error while querying the db: {}", e),
                };
                item.push(field);
            };
            return Ok(item);
        } else {
            return Err(MizeError::new(104)
                .extra_msg("Internal Datastorage Error: the Index of the Item was not found"));
        }
        return Ok(item);
    }
}

pub fn decode(bytes: Vec<u8>) -> Vec<Vec<u8>> {
    let mut count = 0;
    let mut index: Vec<Vec<u8>> = Vec::new();
    let mut key: Vec<u8> = Vec::new();
    loop {
        if bytes[count] == '$' as u8 {
            if key.len() != 0 {
                index.push(key.clone());
                key.clear();
            }
        } else if bytes[count] == '!' as u8 {
            count += 1;
            key.push(bytes[count]);
        } else {
            key.push(bytes[count]);
        }

        if count == bytes.len() -1 {
            index.push(key.clone());
        }

        count += 1;
        if count >= bytes.len() {break};
    }
    return index;
}

pub fn encode(index: Vec<Vec<u8>>) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();
    for key in index {
        let mut count = 0;

        loop {
            if key[count] == '!' as u8 || key[count] == '$' as u8 {
                bytes.insert(count, '!' as u8);
                count += 1;
            }

            count += 1;
            if count >= key.len() {break};
        }
        bytes.push('$' as u8);
        bytes.extend(key);
    }

    return bytes;
}

