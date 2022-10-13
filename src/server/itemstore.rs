
use std::{io, fmt::format, vec};
use surrealdb::{Datastore, Key, Val};
use itertools::Itertools;

pub struct Itemstore {
    db: surrealdb::Datastore,
}

pub type Item = Vec<[Vec<u8>; 2]>;

impl Itemstore {
    pub async fn new(path: String) -> Itemstore {
        let ds = Datastore::new(&(String::from("file://") + &path[..] + "/db")[..]).await.expect("Could not open database");

        let mut tr = ds.transaction(true, false).await.expect("creation of transaction failed");

        match tr.get(vec!('0' as u8)).await {
            Ok(Some(val)) => println!("item 0 already created"),
            Ok(None) => {
                let init = vec!["num_of_items".as_bytes().to_vec(), "next_free_id".as_bytes().to_vec()];
                tr.set("0".to_string().into_bytes(), encode(init)).await;
                let one: u64 = 1;
                let zero: u64 = 0;
                tr.set("0:num_of_items".to_string().into_bytes(), one.to_be_bytes()).await;
                tr.set("0:next_free_id".to_string().into_bytes(), one.to_be_bytes()).await;
                tr.set("0:_commit".to_string().into_bytes(), zero.to_be_bytes()).await;
                tr.set("0:_type".to_string().into_bytes(), "mize-main".as_bytes().to_vec()).await;
                tr.commit().await.expect("error commiting transaction in new function");
            },

            Err(e) => println!("Error while getting item index with id 0: {}", e)
        };
        return Itemstore {db:ds};
    }

    pub async fn create(&self, item: Item) -> u64 {

        let mut tr = self.db.transaction(true, false).await.expect("creation of transaction failed");
        let id = tr.get("0:next_free_id").await
            .expect("no item with id 0 when there definetly should be one")
            .expect("error reading item 0 from db");
        let id_u64 = u64::from_be_bytes(id.clone().try_into().unwrap());
        let mut keys: Vec<Vec<u8>> = Vec::new();

        //set id:_commit
        let key = format!("{}:_commit", id_u64).into_bytes();
        tr.set(key.clone(), vec![0,0,0,0,0,0,0,0]).await;
        keys.push("_commit".to_owned().into_bytes());


        for field in item {
            let mut key = format!("{}:", id_u64).into_bytes();
            key.extend(field[0].clone());
            keys.push(field[0].clone());

            let val = field[1].clone();
            tr.set(key, val).await;
        }
        tr.set(format!("{}", id_u64), encode(keys)).await;

        //increment 0:number_of_items by 1
        let num = tr.get("0:num_of_items".to_string().into_bytes()).await
            .expect("error getting \"0:num_of_items\"")
            .expect("error reading item 0 from db");
        let mut num_u64 = u64::from_be_bytes([num[0], num[1], num[2], num[3], num[4], num[5], num[6], num[7]]) +1;
        tr.set("0:num_of_items".to_string().into_bytes(), num_u64.to_be_bytes()).await;

        //imcrement next_free_id by 1
        let num = tr.get("0:next_free_id".to_string().into_bytes()).await
            .expect("error getting \"0:next_free_id\"")
            .expect("error reading item 0 from db");
        let mut num_u64 = u64::from_be_bytes([num[0], num[1], num[2], num[3], num[4], num[5], num[6], num[7]]) +1;
        tr.set("0:next_free_id".to_string().into_bytes(), num_u64.to_be_bytes()).await;

        let res = tr.commit().await.expect("error commiting transaction in create function");
        return u64::from_be_bytes([id[0], id[1], id[2], id[3], id[4], id[5], id[6], id[7], ])
    }

    pub async fn update(&self, id: u64, new_item: Item){
        let mut tr = self.db.transaction(true, true).await.expect("creation of transaction failed");

        let mut keys: Vec<Vec<u8>> = match tr.get(format!("{}", id).into_bytes()).await {
            Ok(Some(val)) => decode(val),
            Ok(None) => {
                tr.cancel();
                self.create(new_item);
                return;
            },
            Err(e) => panic!("Error while querying the db: {}", e),
        };

        for field in new_item {
            let mut key = id.to_be_bytes().to_vec();
            key.push(':' as u8);
            key.extend(field[0].clone());
            let val = field[1].clone();
            keys.push(field[0].clone());
            tr.set(key, val).await;
        }

        //increment commit number
        let mut commit_key = id.to_be_bytes().to_vec();
        commit_key.extend(":_commit".to_string().into_bytes());

        let mut commit_num: u64 = match tr.get(&*commit_key).await {
            Ok(Some(val)) => u64::from_be_bytes(val.try_into().expect("error converting _commit num to u64")),
            Ok(None) => {
                panic!("item {} has no _commit", id);
            },
            Err(e) => panic!("Error while querying the db: {}", e),
        };

        commit_num += 1;
        tr.set(commit_key, commit_num.to_be_bytes());

        keys = keys.into_iter().unique().collect::<Vec<Vec<u8>>>();
        tr.set(format!("{}", id).into_bytes(), encode(keys)).await;

        let res = tr.commit().await.expect("error commiting transaction in update function");
    }

    pub async fn delete(&self, id: u64){
        let mut tr = self.db.transaction(true, true).await.expect("creation of transaction failed");

        let mut keys: Vec<Vec<u8>> = match tr.get(format!("{}", id).into_bytes()).await {
            Ok(Some(val)) => decode(val),
            Ok(None) => {
                panic!("index of item {} not found", id);
            },
            Err(e) => panic!("Error while querying the db: {}", e),
        };

        for key in keys {
            tr.del(key).await;
        }
        tr.del(format!("{}", id)).await;
        let res = tr.commit().await.expect("error commiting transaction in delete function");
    }

    pub async fn get(&self, id: u64) -> Item {
        let mut tr = self.db.transaction(false, false).await.expect("creation of transaction failed");
        let mut item: Item = Vec::new();

        let mut keys: Vec<Vec<u8>> = match tr.get(format!("{}", id).into_bytes()).await {
            Ok(Some(val)) => decode(val),
            Ok(None) => {
                panic!("index of item {} not found", id);
            },
            Err(e) => panic!("Error while querying the db: {}", e),
        };

        for key in keys {
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
        }
        return item;
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

