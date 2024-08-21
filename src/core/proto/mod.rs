use core::fmt;
use std::io;
use nix::libc::CTRL_CMD_UNSPEC;
use serde::Serialize;

use ciborium::{value::Integer, Value as CborValue};

use crate::{error::{IntoMizeResult, MizeError, MizeResult}, id::MizeId, instance::{self, connection::Connection, Instance}, item::{IntoItemData, ItemData}};

#[derive(Clone)]
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
static CMD_UPDATE: u16 = 2;
static CMD_GIVE: u16 = 3;
static CMD_CREATE: u16 = 4;
static CMD_CREATE_REPLY: u16 = 5;
static CMD_UPDATE_REQUEST: u16 = 6;
static CMD_GET_SUB: u16 = 7;
static CMD_SUB: u16 = 8;

#[derive(Debug)]
pub enum MessageCmd {
    Get,
    Update,
    Give,
    Create,
    CreateReply,
    UpdateRequest,
    GetSub,
    Sub,
}




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

    pub fn new_get_sub(id: MizeId, conn_id: u64) -> MizeMessage {
        let id_path = id.path().into_iter().map(|string| CborValue::Text(string.to_owned())).collect();

        let cmd = (CborValue::Integer(MSG_CMD.into()), CborValue::Integer(CMD_GET_SUB.into()));
        let id = (CborValue::Integer(MSG_ID.into()), CborValue::Array(id_path));
        let value = CborValue::Map(vec![cmd, id]);

        MizeMessage::new(value, conn_id)
    }

    pub fn new_sub(id: MizeId, conn_id: u64) -> MizeMessage {
        let id_path = id.path().into_iter().map(|string| CborValue::Text(string.to_owned())).collect();

        let cmd = (CborValue::Integer(MSG_CMD.into()), CborValue::Integer(CMD_SUB.into()));
        let id = (CborValue::Integer(MSG_ID.into()), CborValue::Array(id_path));
        let value = CborValue::Map(vec![cmd, id]);

        MizeMessage::new(value, conn_id)
    }

    pub fn new_create(conn_id: u64) -> MizeMessage {
        let cmd = (CborValue::Integer(MSG_CMD.into()), CborValue::Integer(CMD_CREATE.into()));
        let value = CborValue::Map(vec![cmd]);

        MizeMessage::new(value, conn_id)
    }

    pub fn new_create_reply(id: MizeId, conn_id: u64) -> MizeMessage {
        let id_path = id.path().into_iter().map(|string| CborValue::Text(string.to_owned())).collect();

        let cmd = (CborValue::Integer(MSG_CMD.into()), CborValue::Integer(CMD_CREATE_REPLY.into()));
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

    pub fn new_update_request(id: MizeId, data: ItemData, conn_id: u64) -> MizeMessage {
        let id_path = id.path().into_iter().map(|string| CborValue::Text(string.to_owned())).collect();

        let cmd = (CborValue::Integer(MSG_CMD.into()), CborValue::Integer(CMD_UPDATE_REQUEST.into()));
        let id = (CborValue::Integer(MSG_ID.into()), CborValue::Array(id_path));
        let data = (CborValue::Integer(MSG_DATA.into()), data.cbor().to_owned());
        let value = CborValue::Map(vec![cmd, id, data]);

        MizeMessage::new(value, conn_id)
    }

    pub fn new_update(id: MizeId, data: ItemData, conn_id: u64) -> MizeMessage {
        let id_path = id.path().into_iter().map(|string| CborValue::Text(string.to_owned())).collect();

        let cmd = (CborValue::Integer(MSG_CMD.into()), CborValue::Integer(CMD_UPDATE.into()));
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
            2 => MessageCmd::Update,
            3 => MessageCmd::Give,
            4 => MessageCmd::Create,
            5 => MessageCmd::CreateReply,
            6 => MessageCmd::UpdateRequest,
            7 => MessageCmd::GetSub,
            8 => MessageCmd::Sub,
            _ => {
                return Err(MizeError::new().msg("error cmd of msg was not a valid command"));
            },
        };

        return Ok(cmd);
    }

    
    pub fn id_str(&mut self) -> MizeResult<Vec<String>> {
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
                    let two: Integer = 2.into();
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

        Ok(vec_string)
    }

    pub fn id(&mut self, instance: &Instance) -> MizeResult<MizeId> {
        let id_str = self.id_str()?;
        return instance.new_id(id_str);
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
            let mut found = false;
            for (key, val) in msg_as_map {
                if let CborValue::Integer(key_int) = key {
                    // number three indicates the data field
                    let three: Integer = 3.into();
                    if key_int == &three {
                        tmp_val = val;
                        found = true;
                    }
                }
            }
            if ! found {
                return Err(MizeError::new().msg("error getting the data value form a msg"));
            }
            tmp_val
        };

        return Ok(data.to_owned().into_item_data());
    }
}


// thanks to: https://stackoverflow.com/a/61768916
struct DisplayWriter<'a, 'b>(&'a mut fmt::Formatter<'b>);

impl<'a, 'b> io::Write for DisplayWriter<'a, 'b> {
    fn write(&mut self, bytes: &[u8]) -> std::result::Result<usize, std::io::Error> {
        
        self.0.write_str(&String::from_utf8_lossy(bytes))
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;

        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::result::Result<(), std::io::Error> { todo!() }
}

impl fmt::Display for MizeMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        let mut err = false;

        match self.to_owned().cmd() {
            Ok(cmd) => {
                writeln!(f, "MizeMessage with cmd: {:?}", cmd);
            },
            Err(e) => {
                writeln!(f, "MizeMessage with unparsable cmd");
                err = true;
            }
        }

        match self.to_owned().id_str() {
            Ok(id) => {
                let id_str = id.join("/");
                writeln!(f, "\tid: {:?}", id_str);
            },
            Err(e) => {
                writeln!(f, "\t msg has no id");
                //err = true;
            }
        }

        match self.to_owned().data() {
            Ok(data) => {
                let value = data.cbor();
                write!(f, "\tdata: ");
                let display_writer = DisplayWriter (f);
                if let Err(e) = value.serialize(&mut serde_json::Serializer::pretty(display_writer))
                    .map_err(|serde_err| std::fmt::Error) {
                        writeln!(f, "serialize err: {:?}", e);
                        err = true;
                }
            },
            Err(e) => {
                writeln!(f, "\t msg has no data");
                //err = true;
            },
        }

        if err {
            write!(f, "Full msg: ");
            let display_writer = DisplayWriter (f);
            self.value.serialize(&mut serde_json::Serializer::pretty(display_writer))
                .map_err(|serde_err| std::fmt::Error).mize_result_msg("serialize error")
                .inspect_err(|e| { e.clone().log(); });
        }

        Ok(())

    }
}

impl fmt::Debug for MizeMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\nMizeMessage: ");
        let display_writer = DisplayWriter (f);
        let value = self.to_owned().value();
        value.serialize(&mut serde_json::Serializer::pretty(display_writer))
            .map_err(|serde_err| std::fmt::Error)
    }
}


