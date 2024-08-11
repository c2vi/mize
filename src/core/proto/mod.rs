use ciborium::{value::Integer, Value as CborValue};

use crate::{error::{MizeError, MizeResult}, id::MizeId, instance::{connection::Connection, Instance}, item::{IntoItemData, ItemData}};

#[derive(Debug, Clone)]
pub struct MizeMessage {
    value: CborValue,
    pub conn_id: u64,
}

// msg items
static MSG_CMD: u16 = 1;
static MSG_ID: u16 = 2;
static MSG_DATA: u16 = 3;

// cmds
static CMD_GET: u16 = 1;
static CMD_SET: u16 = 2;
static CMD_GIVE: u16 = 3;

impl MizeMessage {
    pub fn new(value: CborValue, conn_id: u64) -> MizeMessage {
        MizeMessage { value, conn_id }
    }

    pub fn new_get(id: MizeId, conn_id: u64) -> MizeMessage {
        let id_path = id.path().into_iter().map(|string| CborValue::Text(string.to_owned())).collect();

        let cmd = (CborValue::Integer(MSG_CMD.into()), CborValue::Integer(CMD_GET.into()));
        let id = (CborValue::Integer(MSG_ID.into()), CborValue::Array(id_path));
        let value = CborValue::Map(vec![cmd, id]);

        MizeMessage::new(value, conn_id)
    }

    pub fn new_give(id: MizeId, data: ItemData, conn_id: u64) -> MizeMessage {
        let id_path = id.path().into_iter().map(|string| CborValue::Text(string.to_owned())).collect();

        let cmd = (CborValue::Integer(MSG_CMD.into()), CborValue::Integer(CMD_GIVE.into()));
        let id = (CborValue::Integer(MSG_ID.into()), CborValue::Array(id_path));
        let data = (CborValue::Integer(MSG_DATA.into()), data.cbor().to_owned());
        let value = CborValue::Map(vec![cmd, id, data]);

        MizeMessage::new(value, conn_id)
    }

    pub fn value(self) -> CborValue {
        self.value
    }

    pub fn cmd(&self) -> MizeResult<MessageCmd> {
        // return err, if msg is not a map
        let msg_as_map = match &self.value {
            CborValue::Map(val) => val,
            _ => {
                return Err(MizeError::new().msg("Message was not a map"));
            },
        };

        //check if in this map there is a c, otherwise return err
        let val_of_c = {
            let mut tmp_val = 0;
            for (key, val) in msg_as_map {
                if let CborValue::Integer(key_int) = key {
                    // number one indicates the cmd field
                    let one: Integer = 1.into();
                    if key_int == &one {
                        if let CborValue::Integer(val_int) = val {
                            tmp_val = val_int.to_owned().into();
                        }
                    }
                }
            }
            if tmp_val == 0 {
                return Err(MizeError::new().msg("error getting the cmd value form a msg"));
            }
            tmp_val
        };

        // match on value of c
        let cmd = match val_of_c {
            1 => MessageCmd::Get,
            2 => MessageCmd::Set,
            _ => {
                return Err(MizeError::new().msg("error cmd of msg was not a valid command"));
            },
        };

        return Ok(cmd);
    }

    
    pub fn id(&mut self, instance: &mut Instance) -> MizeResult<MizeId> {
        // return err, if msg is not a map
        let msg_as_map = match &self.value {
            CborValue::Map(val) => val,
            _ => {
                return Err(MizeError::new().msg("Message was not a map"));
            },
        };

        let vec_value = {
            let mut data = &CborValue::Null;
            for (key, val) in msg_as_map {
                if let CborValue::Integer(key_int) = key {
                    // number 2 indicates the id field
                    let two: Integer = 1.into();
                    if key_int == &two {
                        data = val;
                    }
                }
            }
            if data == &CborValue::Null {
                return Err(MizeError::new().msg("error getting the data value form a msg"));
            }
            match data {
                CborValue::Array(inner) => inner,
                _ => {
                    return Err(MizeError::new().msg("Id is not a vector"));
                },
            }
        };

        let mut vec_string: Vec<String> = Vec::new();
        for val in vec_value {
            match val {
                CborValue::Text(text) => {
                    vec_string.push(text.to_owned())
                },
                _ => {
                    return Err(MizeError::new().msg("one in the vec of the id in a msg is not of type Text"));
                },
            }
        }

        let id = instance.id_from_vec_string(vec_string)?;
        Ok(id)
    }

    pub fn data(&mut self) -> MizeResult<ItemData> {
        // return err, if msg is not a map
        let msg_as_map = match &self.value {
            CborValue::Map(val) => val,
            _ => {
                return Err(MizeError::new().msg("Message was not a map"));
            },
        };

        let data = {
            let mut tmp_val = &CborValue::Null;
            for (key, val) in msg_as_map {
                if let CborValue::Integer(key_int) = key {
                    // number three indicates the data field
                    let three: Integer = 3.into();
                    if key_int == &three {
                        tmp_val = val;
                    }
                }
            }
            if tmp_val == &CborValue::Null {
                return Err(MizeError::new().msg("error getting the data value form a msg"));
            }
            tmp_val
        };

        return Ok(data.to_owned().into_item_data());
    }
}

pub enum MessageCmd {
    Get,
    Set,
}




