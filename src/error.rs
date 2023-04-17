
use std::collections::HashMap;
use std::fmt::format;
use std::string::FromUtf8Error;
use toml::Value;
use colored::Colorize;

use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;
use toml;

use crate::server::proto::{self, JsonMessage};
use crate::error::proto::ErrMessage;
use crate::server::proto::MizeMessage;
use crate::server::Peer;

lazy_static::lazy_static! {
    pub static ref ERRORS: HashMap<u32, MizeError> = parse_errors();
}

fn parse_errors() -> HashMap<u32, MizeError>{
    let mut map = HashMap::new();
    let binding = include_str!("../errors.toml")
        .parse::<Value>()
        .expect("Error pasting errors.toml file to Toml.");

    let toml_error = binding.get("error")
        .expect("Error getting error table from errors.toml file for Error handling");

    if let Value::Array(error_arr) = toml_error {
        for err in error_arr {

            let code: u32 = if let Value::Integer(code) = err.get("code")
                .expect("some error object in errors.toml has no code field.") {
                    u32::try_from(*code).expect(&format!("Error converting the error code {} &i64 to u32", code))
                } else {
                    panic!("code value exists in errors.toml, but is not an Integer")
            };

            let message = if let Value::String(message) = err.get("message")
                .unwrap_or(&Value::String("".to_string())) {message.to_string()} else {
                    panic!("message value exists in errors.toml on error obect with code {}, but is not a String", code)
            };

            let category = if let Value::String(category) = err.get("category")
                .expect(&format!("the error object in errors.toml with code {} has no category field", code)) {category.to_string()} else {
                    panic!("category value exists in errors.toml on error obect with code {}, but is not a String", code)
            };

            let error = MizeError{code, message, category, extra_msg: Vec::new(), caused_by_msg: None, code_location: None};

            map.insert(code, error);
        }
    } else {
        panic!("errors.toml File wrong. error path is not an array")
    }

    return map;
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MizeError {
    pub category: String,
    pub code: u32,
    pub message: String,
    pub extra_msg: Vec<String>,
    pub caused_by_msg: Option<CausedByMessage>,
    pub code_location: Option<MizeCodeLocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausedByMessage {
    client_id: u64,
    msg: Box<JsonMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MizeCodeLocation {
    file: String,
    line: u32,
    column: u32,
}

impl From<&std::panic::Location <'_>> for MizeCodeLocation {
    fn from(panic_location: &std::panic::Location) -> MizeCodeLocation {
        let file = panic_location.file().to_string();
        let line = panic_location.line();
        let column = panic_location.column();
        MizeCodeLocation {file, line, column}
    }
}

impl MizeError {
    #[track_caller]
    pub fn new(code: u32) -> MizeError {
        let caller_location = std::panic::Location::caller();

        if let Some(err_ref) = ERRORS.get(&code) {
            let mut err = err_ref.clone();
            err.code_location = Some(caller_location.into());
            return err;
        } else {
            return MizeError {
                category: "error with error system".to_string(),
                code: 114,
                message: format!("The error with code {} was not found in the Errors that where imported from the errors.toml file at build time.", code),
                extra_msg: Vec::new(),
                caused_by_msg: None,
                code_location: Some(caller_location.into()),
            };
        }
    }

    pub fn extra_msg<E>(mut self, msg: E) -> MizeError 
        where E: std::fmt::Display
    {
        self.extra_msg.push(format!("{}", msg));
        return self;
    }
    
    pub fn format(mut self, values: Vec<&str>) -> MizeError {
        let count = 0;
        for val in values {
            self.message += &(" FORMAT_".to_string() + &format!("{}", count) + ": " + val)
        }
        return self;
    }

    fn set_msg(self, msg: MizeMessage, origin: Peer) -> MizeError {
        MizeError::new(11).extra_msg("in set_msg TODO!!")
    }

    fn send(self) -> MizeError{
        self
    }

    fn add_to_err_item(self) -> MizeError {
        self
    }

    fn write_to_stderr(self) -> MizeError {
        eprintln!("[{}] {}", "ERROR".red(), self.to_json());
        self
    }

    pub fn handle(self) -> MizeError {
        return self.send().add_to_err_item().write_to_stderr();
    }

    pub fn is_critical(self){
        panic!("Aborting because of a critical Error")
    }

    pub fn to_json(&self) -> JsonValue{
        if let Ok(string) = serde_json::to_string(self) {
            if let Ok(json) = serde_json::from_str(&string) {
                return json;
            } else {
                return serde_json::json!({
                    "category": "error with error system",
                    "code": 114,
                    "message": "error converting a String to a JsonValue for the ErrMsg",
                })
            }
        } else {
            return serde_json::json!({
                "category": "error with error system",
                "code": 114,
                "message": "error converting an MizeError to String",
            })
        }
    }

    pub fn to_json_message(self) -> JsonMessage {
        proto::JsonMessage::ErrMsg(
            ErrMessage{
                err: self,
            }
        )
    }
}

pub trait MizeResultExtension<T> {
    fn extra_msg<E>(self, msg: E) -> Result<T, MizeError>
        where E: std::fmt::Display
    ;
    fn handle(self) -> Result<T, MizeError>;

    fn is_critical(self) -> T;
}

impl<T> MizeResultExtension<T> for Result<T, MizeError> {
    fn extra_msg<E>(mut self, msg: E) -> Result<T, MizeError>
        where E: std::fmt::Display
    {
        match self {
            Err(ref mut e) => {
                e.extra_msg.push(format!("{}", msg));
                return self;
            },
            Ok(something) => Ok(something),
        }
    }
    fn handle(self) -> Result<T, MizeError> {
        match self {
            Err(e) => {
                Err(e.send().add_to_err_item().write_to_stderr())
            },
            Ok(something) => Ok(something)
        }
    }
    fn is_critical(self) -> T {
        self.expect("Aborting because of a critical Error")
    }
}

impl From<FromUtf8Error> for MizeError {
    #[track_caller]
    fn from(utf_err: FromUtf8Error) -> MizeError{
        let caller_location = std::panic::Location::caller();

        let mut err = MizeError::new(109).extra_msg(format!("From std::string::FromUtf8Error: {}", utf_err));
        err.code_location = Some(caller_location.into());
        return err;
    }
}

impl From<surrealdb::error::Db> for MizeError {
    #[track_caller]
    fn from(sur_err: surrealdb::error::Db) -> MizeError {
        let caller_location = std::panic::Location::caller();

        let mut err = MizeError::new(129).extra_msg(format!("From surrealdb Error: {}", sur_err));
        err.code_location = Some(caller_location.into());
        return err;
    }
}

impl From<serde_json::Error> for MizeError {
    #[track_caller]
    fn from(serde_err: serde_json::Error) -> MizeError {
        let caller_location = std::panic::Location::caller();

        let mut err = MizeError::new(11).extra_msg(format!("From an serde_json::Error: {}", serde_err));
        err.code_location = Some(caller_location.into());
        return err;
    }
}





