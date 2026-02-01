use std::ffi::NulError;
use std::fmt::Display;
use log::{error, info, debug};
use colored::Colorize;

pub type VicResult<T> = Result<T, VicError>;

#[macro_export]
macro_rules! vic_err {
    ($($arg:tt)*) => { VicError::new().msg(format!( $($arg)*)) };
}

#[derive(Debug)]
pub struct VicError {
    pub msg: Vec<String>,
    pub code_location: Option<VicCodeLocation>,
} 

pub trait IntoVicResult<T, S> {
    fn vic_result(self) -> VicResult<T>;
    fn vic_result_msg(self, msg: S) -> VicResult<T> where S: std::fmt::Display ;
}

#[derive(Debug, Clone)]
pub struct VicCodeLocation {
    file: String,
    line: u32,
    column: u32,
}

impl VicError {
    #[track_caller]
    pub fn new() -> VicError {
        let caller_location = std::panic::Location::caller();
        let mut err = VicError { msg: Vec::new(), code_location: None };
        err.location(caller_location.into())
    }
    fn location(mut self, location: VicCodeLocation) -> VicError {
        self.code_location = Some(location);
        self
    }
    pub fn msg<T: Into<String>>(mut self, msg: T) -> VicError {
        self.msg.push(msg.into());
        self
    }
    pub fn log(self) -> VicError {
        error!("VicError envountered!");
        if let Some(ref location) = self.code_location {
            error!("[ {} ] {}", "LOCATION".yellow(), location);
        };
        for msg in &self.msg {
            error!("[ {} ] {}", "MSG".yellow(), msg);
            debug!("[ {} ] {:?}", "MSG".yellow(), msg);
        }
        self
    }
}

////////////////// trait implementations

impl<T: Display> From<T> for VicError {
    #[track_caller]
    fn from(value: T) -> Self {
        let caller_location = std::panic::Location::caller();
        VicError::new()
            .msg(format!("From {}: {}", std::any::type_name_of_val(&value), value))
            .location(caller_location.into())
    }
}

impl<T, E, S> IntoVicResult<T, S> for Result<T, E> where E: std::fmt::Display + std::fmt::Debug {
    #[track_caller]
    fn vic_result(self) -> VicResult<T> {
        let caller_location = std::panic::Location::caller();

        match self {
            Ok(val) => VicResult::Ok(val),
            Err(err) => VicResult::Err(VicError::new()
                .msg(format!("From {}: {}", std::any::type_name_of_val(&err), err))
                .msg(format!("LONG msg: {:?}", err))
                .location(caller_location.into())),
        }
    }
    #[track_caller]
    fn vic_result_msg(self, msg: S) -> VicResult<T> where S: std::fmt::Display {
        let caller_location = std::panic::Location::caller();
        match self {
            Ok(val) => VicResult::Ok(val),
            Err(err) => VicResult::Err(VicError::new()
                .msg(format!("{}", msg))
                .msg(format!("From {}: {}", std::any::type_name_of_val(&err), err))
                .msg(format!("LONG msg: {:?}", err))
                .location(caller_location.into())),
        }
    }
}

impl Display for VicCodeLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "file: {} line: {} column: {}", self.file, self.line, self.column)
    }
}

impl From<&std::panic::Location <'_>> for VicCodeLocation {
    fn from(panic_location: &std::panic::Location) -> VicCodeLocation {
        let file = panic_location.file().to_string();
        let line = panic_location.line();
        let column = panic_location.column();
        VicCodeLocation {file, line, column}
    }
}


