
use std::net::ToSocketAddrs;
use std::{io, fmt::format, vec};
use surrealdb::{Datastore, Key, Val};
use itertools::Itertools;
use crate::error;
use crate::error::MizeError;

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
                let init = vec!["num_of_items".as_bytes().to_vec(), "next_free_id".as_bytes().to_vec()];
                tr.set("0".to_string().into_bytes(), encode(init)).await?;
                let one: u64 = 1;
                let zero: u64 = 0;
                tr.set("0:num_of_items".to_string().into_bytes(), one.to_be_bytes()).await?;
                tr.set("0:next_free_id".to_string().into_bytes(), one.to_be_bytes()).await?;
                tr.set("0:_commit".to_string().into_bytes(), zero.to_be_bytes()).await?;
                tr.set("0:_type".to_string().into_bytes(), "mize-main".as_bytes().to_vec()).await?;
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

        //increment 0:number_of_items by 1
        let num_res = tr.get("0:num_of_items".to_string().into_bytes()).await?;
        if let Some(num) = num_res {
            let mut num_u64 = u64::from_be_bytes([num[0], num[1], num[2], num[3], num[4], num[5], num[6], num[7]]) +1;
            new_id = num_u64;
            tr.set("0:num_of_items".to_string().into_bytes(), num_u64.to_be_bytes()).await?;
        } else {
            return Err(MizeError{
                code: 101,
                kind: "key-missing:item0".to_string(),
                message: "The Item0 should have a key num_of_items, but that is not there.".to_string(),
            });
        }

        //imcrement next_free_id by 1
        let num_res = tr.get("0:next_free_id".to_string().into_bytes()).await?;
        if let Some(num) = num_res {
            let mut num_u64 = u64::from_be_bytes([num[0], num[1], num[2], num[3], num[4], num[5], num[6], num[7]]) +1;
            tr.set("0:next_free_id".to_string().into_bytes(), num_u64.to_be_bytes()).await?;
        } else {
            return Err(MizeError{
                code: 101,
                kind: "key-missing:item0".to_string(),
                message: "The Item0 should have a key next_free_id, but that is not there.".to_string(),
            });
        }

        tr.commit().await?;
        return Ok(new_id)
    }

    pub async fn update(&self, id: u64, new_item: Item) -> Result<(), MizeError>{
        let mut tr = self.db.transaction(true, true).await?;

        let mut keys: Vec<Vec<u8>>  = Vec::new();

        for field in new_item {
            let mut key = format!("{}:", id).into_bytes();
            key.extend(field[0].clone());
            let val = field[1].clone();
            keys.push(field[0].clone());
            tr.set(key, val).await?;
        }

        let keys_res = tr.get(format!("{}", id).into_bytes()).await?;
        if let Some(old_keys) = keys_res {
            keys.extend(decode(old_keys));
            keys = keys.into_iter().unique().collect::<Vec<Vec<u8>>>();
            tr.set(format!("{}", id).into_bytes(), encode(keys)).await?;
        } else {
            return Err(MizeError{
                code: 104,
                kind: "data_storage::index_not_found".to_string(),
                message: "Internal Datastorage Error: the Index of the Item was not found".to_string(),
            });
        }

        //increment commit number
        let mut commit_key = format!("{}:_commit", id).into_bytes();

        let commit_res = tr.get(&*commit_key).await?;
        if let Some(commit_vec) = commit_res {
            let val: [u8;8] = match commit_vec.try_into() {
                Ok(val) => val,
                Err(_) => {
                    return Err(MizeError{
                        code: 105,
                        kind: "don't know yet".to_string(),
                        message: "_commit of this item is no valid u64 (not 8 bytes long)".to_string(),
                    });
                },
            };
            let mut commit_num = u64::from_be_bytes(val);
            commit_num += 1;
            tr.set(commit_key, commit_num.to_be_bytes()).await?;
        } else {
            return Err(MizeError{
                code: 103,
                kind: "key_missing::_commit".to_string(),
                message: "This item has no _commit key, which every item must have.".to_string(),
            });
        }

        tr.commit().await?;
        return Ok(());
    }



    pub async fn delete(&self, id: u64) -> Result<(), error::MizeError>{
        let mut tr = self.db.transaction(true, true).await?;

        let keys_res = tr.get(format!("{}", id).into_bytes()).await?;
        if let Some(keys) = keys_res {
            for key in decode(keys) {
                tr.del(key).await?;
            }
        } else {
            return Err(MizeError{
                code: 104,
                kind: "data_storage::index_not_found".to_string(),
                message: "Internal Datastorage Error: the Index of the Item was not found".to_string(),
            });
        }

        tr.del(format!("{}", id)).await?;

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
            return Err(MizeError{
                code: 104,
                kind: "data_storage::index_not_found".to_string(),
                message: "Internal Datastorage Error: the Index of the Item was not found".to_string(),
            });
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

