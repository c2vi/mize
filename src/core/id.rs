use core::fmt;
use std::{fmt::write, hash::Hash};


#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct MizeId {
    path: Vec<String>
}

impl MizeId {
    pub fn store_part(&self) -> &str {
        self.path.first()
            .expect("an empty MizeId found, that should absolutely not be possible!!!!").as_str()
    }
    pub fn as_vec(self) -> Vec<String> {
        self.path
    }
}

impl fmt::Display for MizeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path.join("/"))
    }
}

impl<T: Into<String>> From<T> for MizeId {
    fn from(value: T) -> Self {
        let string = value.into();
        let path: Vec<String> = string.split("/").map(|v| v.to_owned()).collect();
        return MizeId { path }
    }
}


