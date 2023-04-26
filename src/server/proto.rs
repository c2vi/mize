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
    Pong(),
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

#[serde(from = "SerdeDelta")]
#[serde(into = "SerdeDelta")]
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Delta {
    pub delta: Vec<(Path, Option<JsonValue>)>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SerdeDelta {
    pub delta: Vec<SerdeDeltaEntry>,
}

impl From<SerdeDelta> for Delta {
    fn from(serde: SerdeDelta) -> Delta {
        let mut delta = Delta {delta: Vec::new()};
        for SerdeDeltaEntry(path, json) in serde.delta {
            delta.delta.push((path, json));
        }
        return delta;
    }
}

impl From<Delta> for SerdeDelta {
    fn from(delta: Delta) -> SerdeDelta {
        let mut serde = SerdeDelta {delta: Vec::new()};
        for (path, json) in delta.delta {
            serde.delta.push(SerdeDeltaEntry(path, json));
        }
        return serde;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SerdeDeltaEntry (
        pub Path,
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub Option<JsonValue>,
    );

pub type Path = Vec<String>;

pub type Update = Vec<(MizeId, Delta)>;


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

//mod serde_delta {
//    pub fn serialize(delta: &super::Delta, ser: S) -> Result<S::Ok, S::Error> 
//        where S: Serializer
//    {
//        let delta_seq = ser.serialize_seq(delta.len())?;
//        for (path, value) in delta {
//            let path_seq = ser.serialize_seq(path.len())?;
//            for path_el in path {
//                path_seq.serialize_element(path_el)
//            }
//            path_seq.end();
//            tuple_seq = ser.serialize_seq();
//        }
//        delta_seq.end()
//    }
//    pub fn deserialize<'de>(deser: D) -> Result<super::Delta, D::Error>
//        where D: Deserializer
//    {
//    }
//}

//###//===================================================
//impls

//impl Serialize for Delta {
//    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//        where
//            S: serde::Serializer 
//    {

//        let delta = self.delta;
//        let id_string = match self {
//            MizeId::Module { mod_name, id } => {String::from("#") + mod_name + id},
//            MizeId::Upstream(id) => id.to_owned(),
//            MizeId::Local(num) => {format!("{}", num)},
//            MizeId::None => {"".to_owned()},
//        };
//        serializer.serialize_str(&id_string)
//    }
//}

//impl<'de> Deserialize<'de> for Delta {
//    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//    where
//        D: Deserializer<'de>,
//    {
//        #[derive(Deserialize)]
//        #[serde(field_identifier, rename_all = "lowercase")]
//        enum Field { Delta }

        // This part could also be generated independently by:
        //
        //    #[derive(Deserialize)]
        //    #[serde(field_identifier, rename_all = "lowercase")]
        //    enum Field { Secs, Nanos }
//        impl<'de> Deserialize<'de> for Field {
//            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
//            where
//                D: Deserializer<'de>,
//            {
//                struct FieldVisitor;

//                impl<'de> Visitor<'de> for FieldVisitor {
//                    type Value = Field;

//                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//                        formatter.write_str("`secs` or `nanos`")
//                    }

//                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
//                    where
//                        E: de::Error,
//                    {
//                        match value {
//                            "secs" => Ok(Field::Secs),
//                            "nanos" => Ok(Field::Nanos),
//                            _ => Err(de::Error::unknown_field(value, FIELDS)),
//                        }
//                    }
//                }

//                deserializer.deserialize_identifier(FieldVisitor)
//            }
//        }

//        struct DeltaVisitor;

//        impl<'de> Visitor<'de> for DeltaVisitor {
//            type Value = Delta;

//            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//                formatter.write_str("struct Delta")
//            }

//            fn visit_seq<V>(self, mut seq: V) -> Result<Duration, V::Error>
//            where
//                V: SeqAccess<'de>,
//            {
//                let secs = seq.next_element()?
//                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
//                let nanos = seq.next_element()?
//                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
//                Ok(Duration::new(secs, nanos))
//            }

//            fn visit_map<V>(self, mut map: V) -> Result<Delta, V::Error>
//            where
//                V: MapAccess<'de>,
//            {
//                let mut delta = None;
//                while let Some(key) = map.next_key()? {
//                    match key {
//                        Field::Delta => {
//                            if secs.is_some() {
//                                return Err(de::Error::duplicate_field("delta"));
//                            }
//                            secs = Some(map.next_value()?);
                            // do deserialisation of Vec<(Path, ...)>
//                        }
//                    }
//                }
//                let delta = delta.ok_or_else(|| de::Error::missing_field("delta"))?;

//                Ok(Delta::new(secs, nanos))
//            }
//        }

//        const FIELDS: &'static [&'static str] = &["delta"];
//        deserializer.deserialize_struct("Delta", FIELDS, DeltaVisitor)
//    }
//}

//impl<'de> Deserialize<'de> for Delta {
//    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//    where
//        D: serde::Deserializer<'de>,
//    {
//        struct MizeIdVisitor;

//        impl<'de> Visitor<'de> for Delta {
//            type Value = Delta;

//            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
//                formatter.write_str("a Delta. mize::server::proto::Delta")
//            }

//            fn visit_map<E>(self, map: &str) -> Result<Self::Value, E>
//            where
//                E: Error,
//            {
//                let first_char = match v.chars().nth(0) {
//                    Some(ch) => ch,
//                    None => {
//                        return Err(de::Error::invalid_value(
//                                de::Unexpected::Other("hi, what is this Unexpected for????"), &"The id was empty"));
//                    },
//                };

//                if (first_char == '#'){
//                    let iter = v.chars();

                    //scip the #
//                    let mod_name: String = iter.clone().take_while(|ch| ch != &'#' || ch != &'/').collect();
//                    let id: String = iter.collect();
//                    return Ok(MizeId::Module { mod_name, id});
//                } else {
//                    let id: u64 = match v.parse(){
//                        Ok(num) => num,
//                        Err(_) => {
//                            return Err(de::Error::invalid_value(de::Unexpected::Other("hi, what is this Unexpected for????"), &"Could not parse the id to an u64"));
//                        },
//                    };
//                    return Ok(MizeId::Local(id));
//                }
//            }

//            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
//            where
//                A: serde::de::MapAccess<'de>,
//            {

//            }
//        }

//        deserializer.deserialize_map(DeltaVisitor {})
//    }
//}

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

impl Delta {
    pub fn new() -> Delta{
        Delta { delta: Vec::new() }
    }

    pub fn append(&mut self, path: Vec<&str>, json: JsonValue){
        let new_path: Vec<String> = path.into_iter().map(|e| e.to_owned()).collect();
        self.delta.push((new_path, Some(json)));
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
            handle_update(update, origin, mutexes.clone()).await?;

//            let subs = mutexes.subs.lock().await;

//            for (id, delta) in new_update {
                //send that delta to every origin that is subbed the item
//                let peers = match subs.get(&id) {
//                    Some(vec) => vec,
//                    None => continue,
//                };
//                let message = UpdateMessage{id, delta};

//                for peer in peers {
                    //TODO: send should only take a reference
//                    peer.send(message.clone()).await
//                }
//            }
        },


        JsonMessage::Create(msg) => {
            let itemstore = mutexes.itemstore.lock().await;
            let id = itemstore.create(msg.item, mutexes.clone(), origin.clone()).await?;
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

pub async fn handle_update(update: Update, origin: Peer, mutexes: Mutexes) -> Result<(), MizeError> {
    mutexes.update_tx.send((update, origin)).await.map_err(|err| MizeError::new(11).extra_msg(format!("From SendError: {}", err)));
    Ok(())
}

pub async fn update_thread(mut update_rx: mpsc::Receiver<(Update, Peer)>, mutexes: Mutexes) -> Result<(), MizeError> {
    while let Some((update, origin)) = update_rx.recv().await {
        println!("UPDATE: {:?}", update.iter().next().unwrap().0);
        //TODO: run update code from modules and types

        let itemstore = mutexes.itemstore.lock().await;
        let subs = mutexes.subs.lock().await;

        for (id, delta) in update {

            //update the items in the itemstore
            if let MizeIdKind::Local(id) = id.kind()? {
                itemstore.update(id.clone(), delta.clone()).await?;
            } else {
                return Err(MizeError::new(11).extra_msg("updates to non local items are not handeld yet"));
            }

            //send a update msg to subbed clients
            let peers = match subs.get(&id) {
                Some(hi) => hi,
                None => break,
            };
            let msg = UpdateMessage{id, delta};
            for peer in peers {
                peer.send(msg.clone()).await;
            }
        }

    }
    Ok(())
}

