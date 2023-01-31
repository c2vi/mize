
    // 1:   get
    // 2:   give (by default only give the first 1k bytes, and the first 100 fields)
    // 3:   get_val (to get more than 1k bytes)
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
    // 15:  get_and_sub
    // 16:  get_fields
    // 17:  error
    // 18:  give_fields


    // ID
    // just a number:                   localy storeWEK_K2JNQqkd
    // #hello.mize.works#034.312        upstream in the internet ... domain specified
    // !youtube!WEK_K2JNQqk             upstream in a module ... if module is not on local server
    // forwarded to configured upstream server (would happen, if we are running in daemon mode)
 
pub static VERSION: u8 = 1;

pub static MSG_GET: u8 = 1;
pub static MSG_GIVE: u8 = 2;
pub static MSG_GET_VAL: u8 = 3;
pub static MSG_GIVE_VAL: u8 = 4;
pub static MSG_UNAUTHORIZED: u8 = 5;
pub static MSG_SUB: u8 = 6;
pub static MSG_UNSUB: u8 = 7;
pub static MSG_UPDATE_REQUEST: u8 = 8;
pub static MSG_UPDATE_DENY: u8 = 9;
pub static MSG_UPDATE: u8 = 10;
pub static MSG_DELETE: u8 = 11;
pub static MSG_CREATE: u8 = 12;
pub static MSG_CREATED_ID: u8 = 13;
pub static MSG_UNSUPPORTED_VERSION: u8 = 14;
pub static MSG_GET_AND_SUB: u8 = 15;
pub static MSG_GET_FIELDS: u8 = 16;
pub static MSG_ERROR: u8 = 17;
pub static MSG_GIVE_FIELDS: u8 = 18;

use std::time::Duration;
//use std::panic::update_hook;
use std::vec;
use std::collections::HashMap;

use crate::server::itemstore;
use crate::server::Mutexes;
use crate::server::itemstore::encode;
use crate::error;
use crate::server::Client;
use std::sync::Arc;
use futures_util::stream::Collect;
use tokio::sync::{Mutex, mpsc};
use crate::server;
use crate::error::MizeError;

#[derive(Clone, Debug)]
pub struct Message {
    pub raw: Vec<u8>,
    version: u8,
    cmd: u8,
    origin: Origin,
    index: usize,
    meta_gotten: bool,
    has_meta: bool,
    id: Option<String>,
}

//pub enum Response {
//    One(Vec<u8>),
//    All(Vec<u8>),
//    AllSubbed(String, Vec<u8>),
//    None,
//}

#[derive(Clone, Debug)]
pub enum Origin {
    Client(server::Client), //the client id
    Module(server::Module), //Module_name
    Upstream(String), //a hostname Type, but for now just a string
}

impl Origin {
    pub async fn send(&self, message: Message){
        match self {
            Origin::Client(client) => {client.tx.send(message).await;},
            Origin::Module(module) => {module.tx.send(message).await;},
            Origin::Upstream(_) => {},
        }
    }
    pub fn get_id(&self) -> u64{
        match self {
            Origin::Client(client) => client.id,
            Origin::Module(module) => module.client_id,
            Origin::Upstream(_) => 0,
        }
    }
}

#[derive(Clone)]
pub struct Update {
    pub id: String,
    origin: Origin,
    pub raw: Vec<(Vec<u8>, Delta)>,
}

#[derive(Clone, Debug)]
pub struct Delta {
    raw: Vec<DeltaCmd>,
}

#[derive(Clone, Debug)]
pub enum DeltaCmd {
    Replace(u32, u32, Vec<u8>),
    Insert(u32, u32, Vec<u8>),
    Delete(u32, u32),
}

impl Delta {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Delta, MizeError> {
        let out_of_bytes = Err(MizeError::new(107).extra_msg("There were not eneough Bytes in the message to complete a Delta."));
        let mut bytes = bytes.iter();
        let mut delta = Delta{raw: Vec::new()};

        while true {
            let cmd = if let Some(cmd) = bytes.next() {
                cmd
            } else {
                //end of bytes
                return Ok(delta);
            };

            match cmd {
                0 => {
                    let mut start_bytes: [u8; 4] = [0,0,0,0];
                    for i in 0..4 {start_bytes[i] = if let Some(byte) = bytes.next(){*byte} else {return out_of_bytes;}};
                    let start = u32::from_be_bytes(start_bytes);

                    let mut stop_bytes: [u8; 4] = [0,0,0,0];
                    for i in 0..4 {stop_bytes[i] = if let Some(byte) = bytes.next(){*byte} else {return out_of_bytes;}};
                    let stop = u32::from_be_bytes(stop_bytes);

                    let mut val: Vec<u8> = Vec::new();
                    for i in 0..stop-start {val.push(if let Some(byte) = bytes.next(){*byte} else {return out_of_bytes;})};

                    delta.raw.push(DeltaCmd::Replace(start, stop, val));
                },
                1 => {
                    let mut start_bytes: [u8; 4] = [0,0,0,0];
                    for i in 0..4 {start_bytes[i] = if let Some(byte) = bytes.next(){*byte} else {return out_of_bytes;}};
                    let start = u32::from_be_bytes(start_bytes);

                    let mut stop_bytes: [u8; 4] = [0,0,0,0];
                    for i in 0..4 {stop_bytes[i] = if let Some(byte) = bytes.next(){*byte} else {return out_of_bytes;}};
                    let stop = u32::from_be_bytes(stop_bytes);

                    let mut val: Vec<u8> = Vec::new();
                    for i in 0..stop-start {val.push(if let Some(byte) = bytes.next(){*byte} else {return out_of_bytes;})};

                    delta.raw.push(DeltaCmd::Insert(start, stop, val));
                },
                2 => {
                    let mut start_bytes: [u8; 4] = [0,0,0,0];
                    for i in 0..4 {start_bytes[i] = if let Some(byte) = bytes.next(){*byte} else {return out_of_bytes;}};
                    let start = u32::from_be_bytes(start_bytes);

                    let mut stop_bytes: [u8; 4] = [0,0,0,0];
                    for i in 0..4 {stop_bytes[i] = if let Some(byte) = bytes.next(){*byte} else {return out_of_bytes;}};
                    let stop = u32::from_be_bytes(stop_bytes);

                    delta.raw.push(DeltaCmd::Delete(start, stop));
                },
                _ => {
                    return Err(MizeError::new(107)
                        .extra_msg(&format!("The update message included a DeltaCmd of {}.\
                                 Only 0,1,2 (for: replace, insert, delete) are allowed.", cmd)));
                },
            }
        };
        return Ok(delta);
    }
}

impl Update {
    pub async fn change_whole_val(id: String, key: Vec<u8>, new_val: Vec<u8>, origin: Origin, mutexes: Mutexes) -> Result<Update, MizeError> {
        let mut old_val_len: u32 = 0;
        let itemstore = mutexes.itemstore.lock().await;

        let id_u64 = match id.parse::<u64>(){
            Ok(id) => id,
            Err(err) => {
                return Err(MizeError::new(11)
                    .extra_msg("Error while parsing id to u64 (std::num::ParseIntError) in order to get it from the itemstore.\
                        (get or get_sub message)"));
            }
        };

        let item = itemstore.get(id_u64).await?;

        for field in item {
            if (field[0] == key){
                old_val_len = field[1].len() as u32;
            }
        }

        let delete_all = DeltaCmd::Delete(0, old_val_len);
        let replace_all = DeltaCmd::Replace(0, new_val.len() as u32, new_val);
        let deltas = Delta{raw: vec![delete_all, replace_all]};
        let update: Update = Update { id, raw: vec![(key, deltas)], origin};
        return Ok(update);
    }

    pub fn to_message(self, origin: Origin) -> Message{
        return Message::from_update(self, origin);
    }

    pub fn from_message(mut message: Message) -> Result<Update, MizeError> {

        message.index = 2;
        message.id = None;

        let id = message.get_id()?;

        let mut update = Update{id, raw: Vec::new(), origin: message.clone().origin};

        let num_of_updates = message.get_u32();

        for i in 0..num_of_updates as usize {
            let key_len = message.get_u32();
            let key = message.get_bytes(key_len as usize);

            let update_len = message.get_u32();
            let update_bytes = message.get_bytes(update_len as usize);

            //get DeltaCmds
            let delta = Delta::from_bytes(update_bytes)?;

            update.raw.push((key, delta));
        }
        return Ok(update);

    }
}

impl Message {
    pub fn from_bytes(vec: Vec<u8>, origin: Origin) -> Message {

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

            if count == 2 {break};
        };
        
        let mut has_meta = false;
        if (cmd >= 128) {cmd = cmd - 128; has_meta = true}

        Message { raw: vec, version, cmd, index: 2 , origin, meta_gotten: false, id: None, has_meta}
    }
    pub async fn send(self, origin: &Origin) {
        origin.send(self).await;
    }

    pub async fn forward_module(mut self, id: String, mutexes: Mutexes) -> Result<(), MizeError>{
        let mut meta: HashMap<String, String> = HashMap::new();
        meta.insert("c".to_string(), format!("{}", self.origin.get_id()));
        self.set_metadata(meta)?;

        //add 128 to the cmd
        self.raw[1] = self.raw[1] + 128;


        let mut id_iter = id.chars();
        //scip the first "!"
        id_iter.next();
        let mod_name = id_iter.take_while(|&ch| ch != '!').collect::<String>();

        let modules = mutexes.modules.lock().await;
        for module in &*modules {
            if (module.name == mod_name) {
                self.send(&Origin::Module(module.clone())).await;
                return Ok(());
            }
        }
        return Ok(());
    }

    pub fn from_update(update: Update, origin: Origin) -> Message{
        let mut message: Vec<u8> = vec![VERSION, MSG_UPDATE];

        // id
        message.extend(update.id.into_bytes());
        message.push("/".to_string().into_bytes()[0]);

        // update_len
        let update_len = update.raw.len() as u32;
        message.extend(update_len.to_be_bytes());

        for update in update.raw {
            //key_len
            let key_len = update.0.len() as u32;
            message.extend(key_len.to_be_bytes());
            
            // key
            message.extend(update.0);

            //delta_len
            let mut delta_len: u32 = 0;

            // deltas
            let mut delta_part: Vec<u8> = Vec::new();

            // loop
            for delta in update.1.raw {
                match delta {
                    DeltaCmd::Replace(start, stop, data) => {
                        delta_len += 9; //for len of start, stop and cmd
                        delta_len += data.len() as u32;
                        delta_part.push(0);
                        delta_part.extend(start.to_be_bytes());
                        delta_part.extend(stop.to_be_bytes());
                        delta_part.extend(data)
                    },
                    DeltaCmd::Insert(start, stop, data) => {
                        delta_len += 9; //for len of start, stop and cmd
                        delta_len += data.len() as u32;
                        delta_part.push(1);
                        delta_part.extend(start.to_be_bytes());
                        delta_part.extend(stop.to_be_bytes());
                        delta_part.extend(data)
                    },
                    DeltaCmd::Delete(start, stop) => {
                        delta_len += 9; //for len of start, stop and cmd
                                        //
                        delta_part.push(2);
                        delta_part.extend(start.to_be_bytes());
                        delta_part.extend(stop.to_be_bytes());
                    },
                }
            };

            message.extend(delta_len.to_be_bytes());
            message.extend(delta_part);

        }
        return Message::from_bytes(message, origin);
    }

    pub fn set_metadata(&mut self, metadata: HashMap<String, String>) -> Result<(), MizeError> {
        let mut meta_vec: Vec<u8> = Vec::new();
        meta_vec.push('{' as u8);

        for (mut key, mut val) in metadata.iter(){
            meta_vec.append(&mut key.clone().into_bytes());
            meta_vec.push('=' as u8);
            meta_vec.append(&mut val.clone().into_bytes());
            meta_vec.push(';' as u8);
        }
        //meta_vec.remove(meta_vec.len());
        meta_vec.push('}' as u8);

        self.raw.splice(2..2, meta_vec);
        return Ok(());
    }

    pub fn get_metadata(&mut self) -> Result<HashMap<String, String>, MizeError> {
        if !self.has_meta {
            return Err(MizeError::new(108).extra_msg("The message should have medatata, but it hasn't. (cmd <= 128)"))
        };

        let meta_err = MizeError::new(107).extra_msg("Error while parsing the metadata of a message.");

        let mut meta_vec:Vec<u8> = Vec::new();
        let mut msg_iter = self.raw[self.index..].iter();

        if (msg_iter.nth(0).unwrap() != &('{' as u8)){
            return Err(meta_err);
        };

        for ch in msg_iter {

            if (ch == &('}' as u8)){
                let mut map: HashMap<String, String> = HashMap::new();
                let string = String::from_utf8(meta_vec)?;
                let sp: Vec<&str> = string.split(";").collect();

                for st in sp.clone() {
                    if (st == ""){continue;}
                    let mut split = st.split("=").collect::<Vec<&str>>();
                    let key = split.clone().into_iter().nth(0).ok_or(meta_err.clone())?;
                    let val = split.into_iter().nth(1).ok_or(meta_err.clone())?;
                    map.insert(key.to_string(), val.to_string());
                }
                self.meta_gotten = true;
                return Ok(map);
            }
            meta_vec.push(*ch);
        };

        return Err(MizeError::new(107).extra_msg("Error while parsing the metadata of a message."))

    }

    pub fn get_id(&mut self) -> Result<String, MizeError> {
        //get the metadata just to increment the index
        if let Some(id) = &self.id {
            return Ok(id.clone());
        }
        if (self.has_meta && !self.meta_gotten){
            self.get_metadata();
        }
        let mut ch: u8 = 0;
        let mut id = String::new();
        for ch in &self.raw[self.index..] {

            if (*ch == 47 as u8){
                self.index += 1;
                self.id = Some(id.clone());
                return Ok(id);
            } else {
                id.push(*ch as char);
                self.index += 1;
            }
        }

        return Err(MizeError::new(112).extra_msg("There is no / to indicate where the id ends"));

//        let len = self.raw.len();
//        while self.index < 10000 {
//            if (self.index == len){
//                return Err(MizeError {
//                    code: 112,
//                    kind: "faulty_message".to_string(),
//                    message: "There is no / to indicate where the id ends".to_string(),
//                })
//            }
//            let ch: u8 = self.raw[self.index];
//            if (ch == 47) {break};

//            self.index += 1;
//        }
        //skip "/"
//        self.index += 1;

//        self.id_gotten = true;
//        return Ok(id);
    }

    pub fn get_u32(&mut self) -> u32 {
            let tmp = &self.raw[self.index..self.index + 4];
            let num: u32 = u32::from_be_bytes([tmp[0], tmp[1], tmp[2], tmp[3]]);
            self.index += 4;

            return num;
    }

    pub fn get_u64(&mut self) -> u64 {
            let tmp = &self.raw[self.index..self.index + 8];
            let num: u64 = u64::from_be_bytes([tmp[0], tmp[1], tmp[2], tmp[3], tmp[4], tmp[5], tmp[6], tmp[7]]);
            self.index += 8;

            return num;
    }
    
    pub fn get_bytes(&mut self, n: usize) -> Vec<u8>{
        let tmp = &self.raw[self.index..self.index + n];
        self.index += n;

        return tmp.to_vec();
    }

}


pub async fn handle_mize_message(
        mut message: Message,
        mutexes: Mutexes,
    ) -> Result<(), MizeError> {


    //###########################################################//
    //get and get_and_sub cmd from client or module
    if ((message.cmd == MSG_GET || message.cmd == MSG_GET_AND_SUB) &&
        (matches!(&message.origin, Origin::Client(_)) || matches!(&message.origin, Origin::Module(_))))
    {

        let id: String = message.get_id()?;
        let client_id: u64 = match message.origin {
            Origin::Client(ref Client) => Client.id,
            Origin::Module(ref Module) => Module.client_id,
            Origin::Upstream(_) => 0,
        };

        //sub to item in case of MSG_GET_AND_SUB
        if (message.cmd == MSG_GET_AND_SUB) {
            let mut subs = mutexes.subs.lock().await;
            if let Some(mut vec_ids) = subs.get_mut(&id) {
                vec_ids.push(message.origin.clone());
            } else {
                subs.insert(id.clone(), vec![message.origin.clone()]);
            };
            drop(subs);
        };

        //forward to module in case the item is from a Module
        //append medatata
        let first_char = id.chars().nth(0)
            .ok_or(MizeError::new(11).extra_msg("Somehow the id is empty. while handling a get or get_sub message."))?;

        if (first_char == '!'){
            message.clone().forward_module(id.clone(), mutexes.clone()).await;
            return Ok(());
        }

        //forward to upstream server in case the item is from another Server
        if (first_char == '#'){
            return Ok(());
        }


        // answer:
        // u8: version
        // u8: cmd (2 for give)
        // id: terminated by "/"
        // u32: num_of_fields
        // as often as num_of_fields:
            // u64: key_len
            // key_len: key
            // u64: val_len
            // val_len: val
        let mut answer: Vec<u8> = vec![VERSION, MSG_GIVE];
        answer.extend(format!("{}/", id).into_bytes());

        let itemstore = mutexes.itemstore.lock().await;
        
        let id_u64 = match id.parse::<u64>(){
            Ok(id) => id,
            Err(err) => {
                return Err(MizeError::new(11)
                    .extra_msg("Error while parsing id to u64 in order to get it from the itemstore.\
                        (get or get_sub message)"));
            }
        };

        let mut item = match itemstore.get(id_u64).await {
            Ok(item) => item,
            Err(err) => {return Err(err)},
        };

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

        let msg = Message::from_bytes(answer, message.origin.clone());
        message.origin.send(msg).await;
        return Ok(());
    };


    //###########################################################//
    //give from module
    if (message.cmd == MSG_GIVE && matches!(&message.origin, Origin::Module(_))){
        let meta = message.get_metadata()?;
        let id: String = message.get_id()?;
        let client_id_str = meta.get("c")
            .ok_or(MizeError::new(11)
            .extra_msg("there is no \"c\" key in the metadata of a message gotten from a module"))?;

        let client_id: u64 = client_id_str.parse()
            .map_err(|_| MizeError::new(11)
            .extra_msg("Error (std::num::ParseIntError) while parsing the \"c\" key \
                in the Message metadata from a give Message from a Module"))?;

        let clients = mutexes.clients.lock().await;
        let modules = mutexes.modules.lock().await;
        let module: Vec<&server::Module> = modules.iter().filter(|&module| module.client_id == client_id).collect();
        let client: Vec<&Client>= clients.iter().filter(|&client| client.id == client_id).collect();

        //let origin = if let Some(module) = modules.get(module_position)


        let origin = if (module.len() == 1) {
            Origin::Module(module[0].to_owned())
        } else if (client.len() == 1) {
            Origin::Client(client[0].to_owned())
        } else {
            return Err(MizeError::new(11))
        };

        drop(clients);
        drop(modules);

        while message.raw[2] != '}' as u8 {
            println!("CHAR: {:?}", message.raw[2]);
            message.raw.remove(2);
        }
        message.raw.remove(2);
        println!("message raw: {:?}", message.raw);

        message.has_meta = false;
        message.raw[1] = message.raw[1] - 128;

        origin.send(message).await;


        return Ok(());
    };


    //###########################################################//
    //update from module
    if (message.cmd == MSG_UPDATE && matches!(&message.origin, Origin::Module(_))){
        let id: String = message.get_id()?;

        let subs = mutexes.subs.lock().await;

        let empty_vec = &Vec::new();
        let subbed_origins = subs.get(&id).unwrap_or(empty_vec);
        for origin in subbed_origins {
            message.clone().send(origin).await;
        }
        return Ok(());
    };


    //###########################################################//
    //update_request from module and client
    if (message.cmd == MSG_UPDATE_REQUEST && (matches!(&message.origin, Origin::Module(_)) || matches!(&message.origin, Origin::Client(_)))){

        let id: String = message.get_id()?;
        let first_char = id.chars().nth(0)
            .ok_or(MizeError::new(11)
            .extra_msg("Somehow the id is empty. while handling a get or get_sub message."))?;

        if (first_char == '!'){
            message.clone().forward_module(id, mutexes.clone()).await;
        }

        let update = Update::from_message(message.clone())?;
        handle_update(update, mutexes, message.origin.clone()).await?;
        return Ok(());
    };


    //###########################################################//
    //delete from Client or Module
    if (message.cmd == MSG_DELETE && (matches!(&message.origin, Origin::Module(_)) || matches!(&message.origin, Origin::Client(_)))){
        let id: String = message.get_id()?;
        if id == "0".to_string() {
            let err = MizeError::new(100);
            return Err(err);
        }
        return Ok(());
    }


    //###########################################################//
    //create from module and Client
    if (message.cmd == MSG_CREATE && (matches!(&message.origin, Origin::Module(_)) || matches!(&message.origin, Origin::Client(_)))){

        let num_of_fields = message.get_u32();
        let mut item: Vec<[Vec<u8>; 2]> = Vec::new();

        let mut index = 6;

        for i in 0..num_of_fields {
            let key_len = message.get_u32();
            let key = message.get_bytes(key_len as usize);

            let val_len = message.get_u32();
            let val = message.get_bytes(val_len as usize);
            
            item.push([key, val]);
        }

        let itemstore = mutexes.itemstore.lock().await;
        let new_id = itemstore.create(item).await?;

        let mut item0: Vec<[Vec<u8>; 2]> = Vec::new();
        match itemstore.get(0).await {
            Err(err) => {return Err(err);},
            Ok(item) => {item0 = item;},
        }
        println!("got create message");

        //update item0
        let mut next_free_id: Vec<u8> = Vec::new();
        let mut num_of_items: Vec<u8> = Vec::new();
        for field in item0 {
            if field[0] == "next_free_id".to_string().into_bytes() {
                // soooooo ugly
                let mut tmp = u64::from_be_bytes(field[1].clone().try_into().unwrap());
                tmp += 1;
                next_free_id = tmp.to_be_bytes().to_vec();

            }
            if field[0] == "num_of_items".to_string().into_bytes() {
                // soooooo ugly
                let mut tmp = u64::from_be_bytes(field[1].clone().try_into().unwrap());
                tmp += 1;
                num_of_items = tmp.to_be_bytes().to_vec();
            }
        }

        //because Update::change_whole_val trys to lock the itemstore as well
        drop(itemstore);

        let update_next_id: Update = Update::change_whole_val("0".to_string(), "next_free_id".to_string().into_bytes(), next_free_id.clone(), message.origin.clone(), mutexes.clone()).await?;
        let mut update: Update = Update::change_whole_val("0".to_string(), "num_of_items".to_string().into_bytes(), num_of_items.clone(), message.origin.clone(), mutexes.clone()).await?;

        update.raw.extend(update_next_id.raw);

        handle_update(update, mutexes.clone(), message.origin.clone()).await?;

        //send back MSG_CREATED_ID
        let mut created_id_message = vec![VERSION, MSG_CREATED_ID];
        created_id_message.extend(new_id.to_be_bytes());
        message.origin.send(Message::from_bytes(created_id_message, message.origin.clone())).await;
        
        return Ok(());
    }

    //###########################################################//
    //for every other message
    println!("Got a Message That did not Trigger any if Statement....: {:?}", message.raw);
    return Ok(());
}

pub async fn handle_update(mut update: Update, mutexes: Mutexes, origin: Origin) -> Result<(), MizeError>{
    //handle type-code
    //and either call itemstore.update(), send the update to the module or upstream, or spawn another
    //update by calling handle_update()
    

    ///////////////////////////////// TYPE CODE ////////////////////////////////////////
    //can change the update and spawn new updates (to a different item) (which would call the handle_update func again)

    //the "type code" for the Null type (code that effects every item)
    //eg. increment _commit 
    let itemstore = mutexes.itemstore.lock().await;

    let id_u64 = match update.id.clone().parse::<u64>(){
        Ok(id) => id,
        Err(err) => {
            return Err(MizeError::new(11)
                .extra_msg("Error while parsing id to u64 in order to get it from the itemstore.\
                    (get or get_sub message)"));
        }
    };
    let item = itemstore.get(id_u64).await?;
    let commit_pos = item.iter()
        .position(|field| field[0] == "_commit".to_string().into_bytes())
        .ok_or(MizeError::new(103)
        .extra_msg(&format!("Item {} has not _commit", id_u64)))?;

    let cur_commit = u64::from_be_bytes(item[commit_pos][1].clone().try_into()
        .map_err(|_|MizeError::new(105))?
    );

    //because Update::change_whole_val trys to lock the itemstore as well
    drop(itemstore);


    update.raw.extend(Update::change_whole_val(update.id.clone(), "_commit".to_string().into_bytes(), (cur_commit +1).to_be_bytes().to_vec(), update.origin.clone(), mutexes.clone()).await?.raw);


    //run some type-code here
//    if let Some(index) = item.iter().position(|&field| field[0] == "_type".to_string().into_bytes()) {
//        let type_string = String::from_utf8(item[index][0])?;
//        let types = type_string.split(" ").collect::Vec<String>();
//        for &typ[..] in types {
//            match typ {
//                "test-same" => crate::integrated_types::test_same::main(update, item);
//            }
//        };
//    };

    //if update is changing _type, then run type::create(), or type::destroy(), for types that got added or deleted

    //apply item in itemstore
    let itemstore = mutexes.itemstore.lock().await;
    itemstore.update(update.clone()).await?;

    //send update to clients/modules
    let message = update.clone().to_message(origin.clone());
    let subs = mutexes.subs.lock().await;

    let empty_vec = &Vec::new();
    let subbed_origins = subs.get(&update.id).unwrap_or(empty_vec);
    for origin in subbed_origins {
        message.clone().send(origin).await;
    }

    Ok(())
}

// just like all of this crate, definetly could be done with less clones().... and better error
// handling

pub fn apply_delta(mut val: Vec<u8>, delta: Delta) -> Result<Vec<u8>, MizeError>{

    let mut update_iter = delta.raw.iter();
    for delta_cmd in delta.raw.iter() {
        match delta_cmd {
            //r,start:u32,stop:u32,bytes start..stop
            DeltaCmd::Replace(start, stop, bytes) => {
                let bytes = (*bytes).clone();
                let start = *start as usize;
                let stop = *stop as usize;
                let val_len = val.len();
                if (stop > val_len) {
                    val.extend(vec![0; stop-val_len])
                }
                val.splice(start..stop, bytes);
            },
            //i,start:u32, stop:u32, bytes stop-start
            DeltaCmd::Insert(start, stop, bytes) => {
                val.splice((*start as usize)..(*start as usize), (*bytes).clone());
            },
            //d,start:u32,stop:u32
            DeltaCmd::Delete(start, stop) => {
                val.splice((*start as usize)..(*stop as usize), Vec::new());
            },
            _ => {panic!("unknown update command")}
        }
    }

    return Ok(val);
}

