
use std::vec;

use crate::server::itemstore;
use crate::server::itemstore::encode;

use super::itemstore::Itemstore;

pub enum Response {
    One(Vec<u8>),
    All(Vec<u8>),
    None,
}

pub struct Message {
    msg: Vec<u8>,
    version: u8,
    cmd: u8,
    index: usize,
}

impl Message {
    pub fn new(vec: Vec<u8>) -> Message {

        // this weired version does not panic, while others do for some unknown reason  
        let mut count = 0;

        let mut cmd: u8 = 0;
        let mut version: u8 = 0;

        for i in vec.clone() {
            if count == 0 {
                version = i;
            }
            if count == 1 {
                cmd = i
            }

            count += 1;
        };

        Message { msg: vec, version: version, cmd: cmd, index: 2 }
    }

    pub fn get_id(&mut self) -> u64 {
       self.get_u64()
    }

    pub fn get_u32(&mut self) -> u32 {
            let tmp = &self.msg[self.index..self.index + 4];
            let num: u32 = u32::from_be_bytes([tmp[0], tmp[1], tmp[2], tmp[3]]);
            self.index += 4;

            return num;
    }

    pub fn get_u64(&mut self) -> u64 {
            let tmp = &self.msg[self.index..self.index + 8];
            let num: u64 = u64::from_be_bytes([tmp[0], tmp[1], tmp[2], tmp[3], tmp[4], tmp[5], tmp[6], tmp[7]]);
            self.index += 8;

            return num;
    }
    
    pub fn get_bytes(&mut self, n: usize) -> Vec<u8>{
        let tmp = &self.msg[self.index..self.index + n];
        self.index += n;

        return tmp.to_vec();
    }

}

const VERSION: u8 = 1;

pub async fn handle_mize_message(
        mut message: Message,
        itemstore: &itemstore::Itemstore,
    ) -> Response {

    let old_message = message.msg.clone();


    // 1:   get
    // 2:   give
    // 3:   get_val
    // 4:   give_val
    // 5:   unauthorized
    // 6:   sub
    // 7:   unsub
    // 8:   update_request
    // 9:   update_deny
    // 10:  update
    // 11:  delete
    // 12:  create
    // 13:  created_id
    // 14:  unsupported_version

    //println!("message: {:?}", message);
    //let version = message.clone().into_iter().nth(0).expect("message has no 0th byte");
    //let cmd = message.clone().into_iter().nth(1).expect("message has no 1th byte");
    //println!("after chaos");

    // for some weired reason this stupid solution does not panic, while the obove one and vec[0]
    // do
    let mut count = 0;

    let mut cmd: u8 = 0;
    let mut version: u8 = 0;

    for i in old_message.clone() {
        if count == 0 {
            version = i;
        }
        if count == 1 {
            cmd = i
        }

        count += 1;
    };

    //println!("VERSION: {}", version);
    //println!("CMD: {}", cmd);

    match cmd {
        1 => {
            //let id_bytes = *&message[2..9].to_owned().clone();
            let tmp = &old_message[2..10];
            let id: u64 = u64::from_be_bytes([tmp[0], tmp[1], tmp[2], tmp[3], tmp[4], tmp[5], tmp[6], tmp[7]]);

            // answer:
            // u8: version
            // u8: cmd (2 for give)
            // u64: id
            // u32: num_of_fields
            // as often as num_of_fields:
                // u64: key_len
                // key_len: key
                // u64: val_len
                // val_len: val
            let mut answer: Vec<u8> = vec![VERSION, 2];
            answer.extend(id.to_be_bytes());

            let item = itemstore.get(id).await;

            let num_of_fields = item.len() as u32;
            answer.extend(num_of_fields.to_be_bytes());

            for field in item {
                let key_len = field[0].len() as u32;
                answer.extend(key_len.to_be_bytes());
                answer.extend(field[0].clone());
                let val_len = field[1].len() as u32;

                answer.extend(val_len.to_be_bytes());
                answer.extend(field[1].clone());
            }
            return Response::One(answer);
        },
        2 => {return Response::None;},
        3 => {return Response::None;},
        4 => {return Response::None;},
        5 => {return Response::None;},
        6 => {return Response::None;},
        7 => {return Response::None;},
        8 => {
            let mut answer: Vec<u8> = message.msg.clone();
            answer[1] = 10;

            let id = message.get_id();

            let mut item = itemstore.get(id).await;

            let num_of_updates = message.get_u32();

            for i in 0..num_of_updates {
                let key_len = message.get_u32();
                let key = message.get_bytes(key_len as usize);

                let update_len = message.get_u32();
                let update = message.get_bytes(update_len as usize);

                //change the item
                for mut field in item.clone() {
                    if field[0] == key {
                        field[1] = apply_update(field[1].clone(), update.clone());
                    }
                }
            }
            itemstore.update(id, item).await;

            return Response::All(answer);
        },
        9 => {return Response::None;},
        10 => {return Response::None;},

        //delete
        11 => {
            let tmp = &old_message[2..10];
            let id: u64 = u64::from_be_bytes([tmp[0], tmp[1], tmp[2], tmp[3], tmp[4], tmp[5], tmp[6], tmp[7]]);
            if id == 0 {
                return Response::None;
            }

            itemstore.delete(id);
            return Response::None;
        },

        //create
        12 => {
            let tmp = &old_message[2..6];
            let num_of_fields: u32 = u32::from_be_bytes([tmp[0], tmp[1], tmp[2], tmp[3]]);
            let mut item: Vec<[Vec<u8>; 2]> = Vec::new();

            let mut index = 6;
            for i in 0..num_of_fields {
                let mut tmp = &old_message[index..index + 4];
                let key_len = u32::from_be_bytes([tmp[0], tmp[1], tmp[2], tmp[3]]);
                let key = old_message[index + 4..(index + 4 + key_len as usize)].to_vec();

                index += 4 + key_len as usize;

                tmp = &old_message[index..index + 4];
                let val_len = u32::from_be_bytes([tmp[0], tmp[1], tmp[2], tmp[3]]);
                let val = old_message[index + 4..(index + 4 + val_len as usize)].to_vec();
                
                index += 4 + val_len as usize;

                item.push([key, val]);
            }

            itemstore.create(item).await;

            let anser: Vec<u8> = vec![1,1];
            return Response::None;
        },
        13 => {return Response::None;},
        14 => {return Response::None;},
        _ => {return Response::None;},
    }


}

pub fn apply_update(val: Vec<u8>, update: Vec<u8>) -> Vec<u8>{
    Vec::new()
}

