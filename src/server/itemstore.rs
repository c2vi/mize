
use rocksdb::{SingleThreaded, DBCommon, DB, Options};
use std::io;

pub struct itemstore {
    db: DB,
}

impl itemstore {
    pub fn new(path: String) -> itemstore {
        let db = DB::open_default(&path).expect(&format!("Could not open rocksdb database at {}", &path)[..]);

        match db.get(b"0") {
            Ok(Some(val)) => (),
            Ok(None) => {
                //db.set(b"0", )
                println!("hi")
            },
            Err(e) => println!("Error reading item 0 from db: {}", e),
        }
        return itemstore{db:db};
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

