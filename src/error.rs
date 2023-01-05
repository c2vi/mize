
use std::string::FromUtf8Error;

use json;
use crate::server::proto;

#[derive(Debug)]
pub struct MizeError {
    pub kind: String,
    pub code: u32,
    pub message: String,
}

impl MizeError {
    pub fn to_json(self: MizeError) -> json::JsonValue{
        let data = json::object!{
            kind: self.kind,
            code: self.code,
            message: self.message,
        };
        return data;
    }

    pub fn to_message(self: MizeError, origin: proto::Origin) -> proto::Message {
        let data = self.to_json();
        let mut msg = vec![crate::PROTO_VERSION, proto::MSG_ERROR];
        msg.extend(json::stringify(data).into_bytes());
        return proto::Message::from_bytes(msg, origin);
    }
}

impl From<FromUtf8Error> for MizeError {
    fn from(utf_err: FromUtf8Error) -> MizeError{
        let err = MizeError {
            code: 109,
            kind: "faulty_message".to_string(),
            message: "std::string::FromUtf8Error while parsing something from the message".to_string(),
        };
        return err;
    }
}

impl From<surrealdb::Error> for MizeError {
    fn from(sur_err: surrealdb::Error) -> MizeError{
        let cat = format!("data-storage:surrealdb:{:?}", sur_err);
        let msg = format!("Something went wrong internally with the data-storage:surrealdb: {}", sur_err);
        let err = MizeError {
            code: 128,
            kind: cat,
            message: msg,
        };
        return err;
    }
}
