
use std::collections::HashMap;
use std::string::FromUtf8Error;

lazy_static::lazy_static! {
    static ref ERRORS: HashMap<u32, MizeError> = parse_errors();
}

fn parse_errors() -> HashMap<u32, MizeError>{
    let toml_string = include_str!("../errors.toml");
}

use json;
use toml;
use crate::server::proto;


#[derive(Debug, Clone)]
pub struct MizeError {
    pub category: String,
    pub code: u32,
    pub message: String,
    pub extra_msg: Option<String>,
}

impl MizeError {
    pub fn new(code: u32) -> MizeError {
        //let error = ERRORS.

        MizeError {
            category: "error with error system".to_string(),
            code: 114,
            message: "The error with code {} was not found in the errors.toml file".to_string(),
            extra_msg: None
        }
    }

//    pub fn format(&self, values: [str]){
//    }

    pub fn extra_msg(mut self, msg: &str) -> MizeError{
        self.extra_msg = Some(msg.to_string());
        return self;
    }

    pub fn to_json(self: MizeError) -> json::JsonValue{
        if let Some(extra_msg) = self.extra_msg {
            json::object!{
                category: self.category,
                code: self.code,
                message: self.message,
                extra_msg: extra_msg,
            }
        } else {
            json::object!{
                category: self.category,
                code: self.code,
                message: self.message,
            }
        }
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
        MizeError::new(109).extra_msg("std::string::FromUtf8Error while parsing something from the message")
    }
}

impl From<surrealdb::Error> for MizeError {
    fn from(sur_err: surrealdb::Error) -> MizeError{
        MizeError::new(129)
    }
}
