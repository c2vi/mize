use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::format;
use std::string::FromUtf8Error;
use std::io;
use colored::Colorize;
use tracing::{trace, debug, info, warn, error};

#[macro_export]
macro_rules! mize_err {
    ($($arg:tt)*) => { MizeError::new().msg(format!( $($arg)*)) };
}

use crate::proto::MizeMessage;

pub type MizeResult<T> = Result<T, MizeError>;

pub trait MizeResultTrait<T> {
    fn critical(self) -> T;
}

pub trait IntoMizeResult<T, S> {
    fn mize_result(self) -> MizeResult<T>;
    fn mize_result_msg(self, msg: S) -> MizeResult<T> where S: std::fmt::Display ;
}

#[derive(Debug, Clone)]
pub struct MizeError {
    pub category: Vec<String>,
    pub code: u32,
    pub messages: Vec<String>,
    pub caused_by_msg: Option<CausedByMessage>,
    pub code_location: Option<MizeCodeLocation>,
}

#[derive(Debug, Clone)]
pub struct CausedByMessage {
    // peer: &Connection, // TODO: add later
    msg: Box<MizeMessage>,
}

#[derive(Debug, Clone)]
pub struct MizeCodeLocation {
    file: String,
    line: u32,
    column: u32,
}

impl Display for MizeCodeLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "file: {} line: {} column: {}", self.file, self.line, self.column)
    }
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
    pub fn code(code: u32) -> MizeError {
        let caller_location = std::panic::Location::caller();

        get_error_by_code(code, caller_location)
    }

    pub fn set_code(mut self, code: u32) -> MizeError {
        self.code = code;
        return self;
    }

    #[track_caller]
    pub fn new() -> MizeError {
        let caller_location = std::panic::Location::caller();

        return MizeError {
            category: Vec::new(),
            code: 0,
            messages: Vec::new(),
            caused_by_msg: None,
            code_location: Some(caller_location.into()),
        }
    }

    pub fn msg<E>(mut self, msg: E) -> MizeError 
        where E: std::fmt::Display
    {
        self.messages.push(format!("{}", msg));
        return self;
    }

    pub fn category<T>(mut self, category: T) -> MizeError where T: Into<String> {
        self.category.push(category.into());
        return self;
    }
    
    fn send(self) -> MizeError{
        self
    }

    fn add_to_err_item(self) -> MizeError {
        self
    }

    pub fn log(self) -> MizeError {
        error!("MizeError envountered!");
        if let Some(ref location) = self.code_location {
            error!("[ {} ] {}", "LOCATION".yellow(), location);
        };
        for msg in &self.messages {
            error!("[ {} ] {}", "MSG".yellow(), msg);
        }
        self
    }

    pub fn handle(self) -> MizeError {
        return self.send().add_to_err_item().log();
    }

    pub fn critical(self){
        let mut msg_iter = self.messages.iter();
        match msg_iter.next() {
            None => {
                error!("{} MizeError with code: {}", "CRITICAL".red(), self.code);
            },
            Some(msg) => {
                error!("{} MizeError with code: {} - {}", "CRITICAL".red(), self.code, msg);
            },
        }
        for msg in msg_iter {
            error!("{} {}", "MSG".red(), msg)
        }
        if let Some(location) = self.code_location {
            error!("{} {}", "LOCATION".red(), location)
        }
        if let Some(caused_by_msg) = self.caused_by_msg {
            error!("{} {:?}", "CAUSED_BY_MESSAGE".red(), caused_by_msg)
        }
        panic!();
    }

    pub fn location(mut self, location: MizeCodeLocation) -> MizeError {

        self.code_location = Some(location);

        self
    }

}

/*
pub trait MizeResultExtension<T> {
    fn extra_msg<E>(self, msg: E) -> Result<T, MizeError>
        where E: std::fmt::Display
    ;
    fn handle(self) -> Result<T, MizeError>;

    fn is_critical(self) -> T;
}

impl<T> MizeResultExtension<T> for Result<T, MizeError> {
    fn msg<E>(mut self, msg: E) -> Result<T, MizeError>
        where E: std::fmt::Display
    {
        match self {
            Err(ref mut e) => {
                e.msg.push(format!("{}", msg));
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
*/

/*
impl From<FromUtf8Error> for MizeError {
    #[track_caller]
    fn from(utf_err: FromUtf8Error) -> MizeError{
        let caller_location = std::panic::Location::caller();

        let mut err = MizeError::code(109).msg(format!("From std::string::FromUtf8Error: {}", utf_err));
        err.code_location = Some(caller_location.into());
        return err;
    }
}
*/

impl<T: Display> From<T> for MizeError {
    #[track_caller]
    fn from(value: T) -> Self {
        let caller_location = std::panic::Location::caller();
        MizeError::new()
            .msg(format!("From {}: {}", std::any::type_name_of_val(&value), value))
            .location(caller_location.into())
    }
}

impl <T> MizeResultTrait<T> for MizeResult<T> {
    fn critical(self) -> T {
        match self {
            Ok(val) => val,
            Err(err) => {
                err.critical();
                unreachable!()
            },
        }
    }
}

impl<T, E, S> IntoMizeResult<T, S> for Result<T, E> where E: std::fmt::Display {
    #[track_caller]
    fn mize_result(self) -> MizeResult<T> {
        let caller_location = std::panic::Location::caller();

        match self {
            Ok(val) => MizeResult::Ok(val),
            Err(err) => MizeResult::Err(MizeError::new()
                .category("misc")
                .msg(format!("From {}: {}", std::any::type_name_of_val(&err), err))),
        }
    }
    #[track_caller]
    fn mize_result_msg(self, msg: S) -> MizeResult<T> where S: std::fmt::Display {
        let caller_location = std::panic::Location::caller();
        match self {
            Ok(val) => MizeResult::Ok(val),
            Err(err) => MizeResult::Err(MizeError::new()
                .category("misc")
                .msg(format!("{}", msg))
                .msg(format!("From {}: {}", std::any::type_name_of_val(&err), err))),
        }
    }
}


fn get_error_by_code(code: u32, caller_location: &std::panic::Location) -> MizeError {
    let err = MizeError {
        category: Vec::new(),
        code: 0,
        messages: Vec::new(),
        caused_by_msg: None,
        code_location: Some(caller_location.into()),
    };

    match code {
        109 => err.set_code(109).category("decoding").msg("conversion from utf8 failed"),
        108 => err.set_code(108).category("decoding").msg("serde deserialisation failed"),
        _ => err,
    }
}



