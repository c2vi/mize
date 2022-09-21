
use std::{io, fmt::format};
use surrealdb::Datastore;
use itertools::Itertools;

pub struct itemstore {
    db: surrealdb::Datastore,
}

pub type Item = Vec<[String; 2]>;


impl itemstore {
    pub async fn new(path: String) -> itemstore {
        let ds = Datastore::new(&(String::from("file://") + &path[..] + "/db")[..]).await.expect("Could not open database");
        println!("in new fn");

        let mut tr = ds.transaction(true, false).await.expect("creation of transaction failed");

        match tr.get(vec!('0' as u8)).await {
            Ok(Some(val)) => println!("item 0 already created"),
            Ok(None) => {
                let init = vec!(String::from("num_of_items"), String::from("next_free_id"));
                tr.set("0".to_string().into_bytes(), encode(init)).await;
                tr.set("0:num_of_items".to_string().into_bytes(), vec!('1' as u8)).await;
                tr.set("0:next_free_id".to_string().into_bytes(), vec!('1' as u8)).await;
                tr.commit().await;
            },
            Err(e) => println!("Error while getting item index with id 0: {}", e)
        };
        return itemstore {db:ds};
    }

    pub async fn create(&self, item: Item) -> u64 {
        let mut tr = self.db.transaction(true, true).await.expect("creation of transaction failed");
        let id = String::from_utf8(tr.get("0:next_free_id").await
            .expect("no item with id 0 when there definetly should be one")
            .expect("error reading item 0 from db"))
            .expect("error converting from utf-8 to String");
        let mut keys: Vec<String> = Vec::new();

        for field in item {
            let key = format!("{}:{}", id, field[0]);
            let val = field[1].clone();
            keys.push(field[0].clone());
            tr.set(key.into_bytes(), val.into_bytes()).await;
        }
        tr.set(id.clone().into_bytes(), encode(keys)).await;

        //increment 0:number_of_items by 1
        let num = tr.get("0:num_of_items".to_string().into_bytes()).await
            .expect("error getting \"0:num_of_items\"")
            .expect("error reading item 0 from db");
        let mut num_u64 = u64::from_be_bytes([num[0], num[1], num[2], num[3], num[4], num[5], num[6], num[7]]) +1;
        tr.set("0:num_of_items".to_string().into_bytes(), num);

        //imcrement next_free_id by 1
        let num = tr.get("0:next_free_id".to_string().into_bytes()).await
            .expect("error getting \"0:next_free_id\"")
            .expect("error reading item 0 from db");
        let mut num_u64 = u64::from_be_bytes([num[0], num[1], num[2], num[3], num[4], num[5], num[6], num[7]]) +1;
        tr.set("0:next_free_id".to_string().into_bytes(), num);

        tr.commit().await;
        let idvec = id.into_bytes();
        return u64::from_be_bytes([idvec[0], idvec[1], idvec[2], idvec[3], idvec[4], idvec[5], idvec[6], idvec[7], ])
    }

    pub async fn update(&self, id: u64, new_item: Item){
        let mut tr = self.db.transaction(true, true).await.expect("creation of transaction failed");

        let mut keys: Vec<String> = match tr.get(format!("{}", id).into_bytes()).await {
            Ok(Some(val)) => decode(val),
            Ok(None) => {
                tr.cancel();
                self.create(new_item);
                return;
            },
            Err(e) => panic!("Error while querying the db: {}", e),
        };

        for field in new_item {
            let key = format!("{}:{}", id, field[0]);
            let val = field[1].clone();
            keys.push(field[0].clone());
            tr.set(key.into_bytes(), val.into_bytes()).await;
        }

        keys = keys.into_iter().unique().collect::<Vec<String>>();
        tr.set(format!("{}", id).into_bytes(), encode(keys)).await;

        tr.commit().await;
    }

    pub async fn delete(&self, id: u64){
    }

    pub async fn get(&self, id: u64) -> Item {
        let mut tr = self.db.transaction(false, false).await.expect("creation of transaction failed");
        let mut item: Item = Vec::new();

        let mut keys: Vec<String> = match tr.get(format!("{}", id).into_bytes()).await {
            Ok(Some(val)) => decode(val),
            Ok(None) => {
                panic!("index of item {} not found", id);
            },
            Err(e) => panic!("Error while querying the db: {}", e),
        };

        for key in keys {
            let mut field: [String; 2] = [String::new(), String::new()];
            field[0] = key.clone();
            field[1] = match tr.get(format!("{}:{}", id, key)).await {
                Ok(Some(val)) => String::from_utf8(val)
                    .expect("error converting value to utf-8 String"),
                Ok(None) => panic!("a field that should be there wasn't"),
                Err(e) => panic!("Error while querying the db: {}", e),
            };
            item.push(field);
        }
        return item;
    }
}

pub fn decode(bytes: Vec<u8>) -> Vec<String> {
    let mut count = 0;
    let mut index: Vec<String> = Vec::new();
    let mut key: Vec<u8> = Vec::new();
    loop {
        if bytes[count] == '$' as u8 {
            if key.len() != 0 {
                index.push(String::from_utf8(key.clone()).expect("Error decoding key"));
                key.clear();
            }
        } else if bytes[count] == '!' as u8 {
            count += 1;
            key.push(bytes[count]);
        } else {
            key.push(bytes[count]);
        }



        if count == bytes.len() -1 {
            index.push(String::from_utf8(key.clone()).expect("Error decoding key"));
        }

        count += 1;
        if count >= bytes.len() {break};
    }
    return index;
}

pub fn encode(index: Vec<String>) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();
    for key in index {
        let mut b = key.into_bytes();
        let mut count = 0;

        loop {
            if b[count] == '!' as u8 || b[count] == '$' as u8 {
                b.insert(count, '!' as u8);
                count += 1;
            }

            count += 1;
            if count >= b.len() {break};
        }
        bytes.push('$' as u8);
        bytes.append(&mut b);
    }

    return bytes;
}

