use derive_more::From;
use futures_util::stream::select_all::Iter;
use serde::de::Error;
use serde_json::json;
use serde::de::Visitor;
use serde::de;

use std::time::Duration;
//use std::panic::update_hook;
use std::vec;
use std::collections::HashMap;

use crate::server::itemstore;
use crate::server::Mutexes;
use crate::server::Peer;
use crate::error;
use crate::server::Client;
use std::sync::Arc;
use futures_util::stream::Collect;
use tokio::sync::{Mutex, mpsc};
use crate::server;
use crate::error::MizeError;
use crate::server::itemstore::Item;
use crate::server::proto;

use serde_json::Value as JsonValue;
use serde::{Serialize, Deserialize};

//###//===================================================
//all the structs and enums
#[derive(From, Debug, Clone)]
pub enum MizeMessage {
    Json(JsonMessage),
    Bin(BinMessage),
}


#[derive(Debug, Clone)]
pub struct BinMessage {
    pub raw: Vec<u8>,
}

#[derive(From, Debug, Clone, Serialize, Deserialize)]
pub struct ErrMessage {
    //pub cat: String,
    pub err: MizeError,
}

//enum variant and a struct for every cmd a good idea???
#[derive(From, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd")]
pub enum JsonMessage {
    #[serde(rename="err")]
    ErrMsg(ErrMessage),
    #[serde(rename="item.get")]
    Get(GetItemMessage),
    #[serde(rename="item.get-sub")]
    GetSub(GetSubMessage),
    #[serde(rename="item.give")]
    Give(GiveItemMessage),
    #[serde(rename="item.create")]
    Create(CreateItemMessage),
    #[serde(rename="item.created-id")]
    CreatedId(CreatedIdMessage),
    #[serde(rename="item.delete")]
    Delete(DeleteItemMessage),
    #[serde(rename="item.sub")]
    Sub(SubItemMessage),
    #[serde(rename="item.unsub")]
    Unsub(UnsubItemMessage),
    #[serde(rename="item.update-req")]
    UpdateRequest(UpdateRequestMessage),
    #[serde(rename="item.update")]
    Update(UpdateMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetItemMessage {
    //pub cat: String,
    //cmd: String,
    #[serde(flatten)]
    id: MizeId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSubMessage {
    //pub cat: String,
    //cmd: String,
    #[serde(flatten)]
    id: MizeId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GiveItemMessage {
    //pub cat: String,
    //cmd: String,
    #[serde(flatten)]
    id: MizeId,
    item: Item,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateItemMessage {
    //pub cat: String,
    //cmd: String,
    #[serde(flatten)]
    id: MizeId,
    item: Item,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatedIdMessage {
    //pub cat: String,
    //cmd: String,
    #[serde(flatten)]
    id: MizeId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteItemMessage {
    //pub cat: String,
    //cmd: String,
    #[serde(flatten)]
    id: MizeId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubItemMessage {
    //pub cat: String,
    //cmd: String,
    #[serde(flatten)]
    id: MizeId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsubItemMessage {
    //pub cat: String,
    //cmd: String,
    #[serde(flatten)]
    id: MizeId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRequestMessage {
    //pub cat: String,
    //cmd: String,
    #[serde(flatten)]
    id: MizeId,
    #[serde(flatten)]
    delta: Delta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMessage {
    //pub cat: String,
    //cmd: String,
    #[serde(flatten)]
    id: MizeId,
    #[serde(flatten)]
    delta: Delta,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Delta {
    pub delta: Vec<(Path, JsonValue)>,
}

type Path = Vec<String>;

type Update = Vec<(MizeId, Delta)>;


#[derive(Debug, Default, Clone, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub struct MizeId {
    #[serde(rename="id")]
    main: String,
}

pub enum MizeIdKind<'a>{
    Module{mod_name: &'a str, id: &'a str},
    Upstream(&'a server::Peer),
    Local(u64),
}

//###//===================================================
//impls

impl MizeId {
    pub fn new(main: String) -> MizeId {
        MizeId { main }
    }

    pub fn as_string(self) -> String {
        self.main
    }

    pub fn new_local(local_part: &str) -> MizeId {
        MizeId { main: local_part.to_owned() }
    }

    pub fn kind(&self) -> Result<MizeIdKind, MizeError> {
        let first_char = match self.main.chars().nth(0) {
            Some(ch) => ch,
            None => {
                return Err(MizeError::new(11).extra_msg("Couldn't get the first char from a MizeId"));
            },
        };

        if (first_char == '#'){
            let iter = self.main.chars();

            //scip the #
            let mod_name_end = iter.enumerate().find(|&r| r.1 == '#' || r.1 == '/').ok_or(MizeError::new(11))?;

            let mod_name = &self.main[0..mod_name_end.0];
            let id = &self.main[mod_name_end.0..];

            return Ok(MizeIdKind::Module{ mod_name, id});

        } else {
            let id: u64 = match self.main.parse(){
                Ok(num) => num,
                Err(_) => {
                    return Err(MizeError::new(11).extra_msg("Couldn't parse a local id into an integer"))
                },
            };
            return Ok(MizeIdKind::Local(id));
        }
    }

}


//###//===================================================
//From trait implementations, to be able to treat everything as a MizeMessage that can be

impl From<GiveItemMessage> for MizeMessage {
    fn from(give: GiveItemMessage) -> MizeMessage {
        let two: JsonMessage = give.into();
        return two.into();
    }
}

impl From<MizeError> for MizeMessage {
    fn from(err: MizeError) -> MizeMessage {
        let hi: ErrMessage = err.into();
        let two: JsonMessage = hi.into();
        return two.into();
    }
}

impl From<CreatedIdMessage> for MizeMessage {
    fn from(give: CreatedIdMessage) -> MizeMessage {
        let two: JsonMessage = give.into();
        return two.into();
    }
}

impl From<UpdateMessage> for MizeMessage {
    fn from(give: UpdateMessage) -> MizeMessage {
        let two: JsonMessage = give.into();
        return two.into();
    }
}

//###//===================================================
// functions

pub async fn handle_json_msg(msg: JsonMessage, origin: Peer, mutexes: Mutexes) -> Result<(), MizeError>{

    match msg {

        JsonMessage::Get(msg) => {
            match msg.id.kind()? {
                //MizeId::Module { mod_name, id } => {
                //TODO: get type of module with mod_name .... and then either forward msg or call
                //module_get_code
                //},

                MizeIdKind::Local(id) => {
                    let itemstore = mutexes.itemstore.lock().await;
                    let response = GiveItemMessage {
                        id: msg.id,
                        item: itemstore.get(id).await?,
                    };
                    origin.send(response).await;
                },

                _ => {return Err(MizeError::new(11).extra_msg("id type is not implemented"));},
            };
            return Ok(());
        },


        JsonMessage::GetSub(msg) => {

            //sub to item
            let mut subs = mutexes.subs.lock().await;
            if let Some(mut vec_ids) = subs.get_mut(&msg.id) {
                vec_ids.push(origin.clone());
            } else {
                subs.insert(msg.id.clone(), vec![origin.clone()]);
            };

            match msg.id.kind()? {
                //MizeId::Module { mod_name, id } => {
                //TODO: get type of module with mod_name .... and then either forward msg or call
                //module_get_code
                //},

                MizeIdKind::Local(id) => {
                    let itemstore = mutexes.itemstore.lock().await;
                    let response = GiveItemMessage {
                        id: msg.id,
                        item: itemstore.get(id).await?,
                    };
                    origin.send(response).await;
                },

                _ => {return Err(MizeError::new(11).extra_msg("id types not implemented"));},
            }

            return Ok(());
        }


        JsonMessage::UpdateRequest(mut msg) => {
            let update: Update = vec![(msg.id, msg.delta)];
            let new_update = handle_update(update , mutexes.clone(), Some(origin)).await?;

            let subs = mutexes.subs.lock().await;

            for (id, delta) in new_update {
                //send that delta to every origin that is subbed the item
                let peers = match subs.get(&id) {
                    Some(vec) => vec,
                    None => continue,
                };
                let message = UpdateMessage{id, delta};

                for peer in peers {
                    //TODO: send should only take a reference
                    peer.send(message.clone()).await
                }
            }
        },


        JsonMessage::Create(msg) => {
            let itemstore = mutexes.itemstore.lock().await;
            let id = itemstore.create(msg.item, mutexes.clone()).await?;
            let response = CreatedIdMessage {id: MizeId::new(format!("{}", id))};

            origin.send(response).await;
            return Ok(());
        },


        JsonMessage::Delete(msg) => {
            match msg.id.kind()? {
                MizeIdKind::Local(id) => {
                    let itemstore = mutexes.itemstore.lock().await;
                    itemstore.delete(id).await?;
                    return Ok(());
                },
                _ => {return Err(MizeError::new(11).extra_msg("deleteng non Local items not implemented yet"));}
            }
        },


        _ => {
            return Err(MizeError::new(11).extra_msg("ItemMessage cmd not handeld"));
        },
    };
    return Ok(())
}


pub async fn handle_update(update: Update, mutexes: Mutexes, origin: Option<Peer>) -> Result<Update, MizeError> {
    //TODO: run update code from modules and types

    let itemstore = mutexes.itemstore.lock().await;

    for (id, delta) in update.clone() {
        if let MizeIdKind::Local(id) = id.kind()? {
            itemstore.update(id, delta).await?;
        } else {
            return Err(MizeError::new(11).extra_msg("updates to non local items are not handeld yet"));
        }
    }

    Ok(update)
}

