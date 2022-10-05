
use std::vec;

use crate::server::itemstore;
use crate::server::itemstore::encode;

const VERSION: u8 = 1;

async fn handle_mize_message(
    message: Vec<u8>,
    itemstore: itemstore::Itemstore,
    ) -> Option<Vec<u8>>{

    let version = &message[0..1];
    let cmd = &message[1..2];

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
    match cmd[0] {
        1 => {
            let id: u64 = u64::from_be_bytes(message[2..9].try_into().expect("slice with incorrect length"));
            let id_bytes = id.to_be_bytes();
            let mut answer: Vec<u8> = vec![VERSION, 2, id_bytes[0], id_bytes[1], id_bytes[2], id_bytes[3]];
            let item = itemstore.get(id).await;

            let mut temp: Vec<Vec<u8>> = Vec::new();

            for field in item {
                let encoded = encode(field.to_vec());
                temp.push(encoded);
            }


            return Some(answer);
        },
        2 => {return None;},
        3 => {return None;},
        4 => {return None;},
        5 => {return None;},
        6 => {return None;},
        7 => {return None;},
        8 => {return None;},
        9 => {return None;},
        10 => {return None;},
        11 => {return None;},
        12 => {return None;},
        _ => {return None;},
    }
}












