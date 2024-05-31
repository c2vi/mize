use core::fmt;
use std::{collections::binary_heap::Iter, fmt::write, hash::Hash};
use interner::{shared::SharedPool, Pooled, shared::SharedVecString};
use std::collections::hash_map::RandomState;

use crate::instance::{store::Store, Instance};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct MizeId {
    pub path: SharedVecString,
}

impl MizeId {
    pub fn store_part(&self) -> &str {
        self.path.iter().nth(0)
            .expect("an empty MizeId found, that should absolutely not be possible!!!!").as_str()
    }
    pub fn path(&self) -> SharedVecString {
        self.path.clone()
    }
    //pub fn as_slice(&self) -> u8 {
        //self.path
    //}
    //pub fn as_iter(&self) -> u8 {
        //self.path.iter()
    //}
}

impl fmt::Display for MizeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path.join("/"))
    }
}

pub trait IntoMizeId<S: Store> {
    fn to_mize_id(self, instance: &Instance<S>) -> MizeId;
}

impl<S: Store> IntoMizeId<S> for &str {
    fn to_mize_id(self, instance: &Instance<S>) -> MizeId {
        instance.id_from_string(self.to_owned())
    }
}

impl<S: Store> IntoMizeId<S> for String {
    fn to_mize_id(self, instance: &Instance<S>) -> MizeId {
        instance.id_from_string(self)
    }
}
impl<S: Store> IntoMizeId<S> for &String {
    fn to_mize_id(self, instance: &Instance<S>) -> MizeId {
        instance.id_from_string(self.to_owned())
    }
}
impl<S: Store> IntoMizeId<S> for Vec<String> {
    fn to_mize_id(self, instance: &Instance<S>) -> MizeId {
        instance.id_from_vec_string(self)
    }
}
impl<S: Store> IntoMizeId<S> for &[&str] {
    fn to_mize_id(self, instance: &Instance<S>) -> MizeId {
        let mut vec: Vec<String> = Vec::new();
        for i in self {
            vec.push((*i).to_owned())
        }
        instance.id_from_vec_string(vec)
    }
}
impl<S: Store> IntoMizeId<S> for Vec<&str> {
    fn to_mize_id(self, instance: &Instance<S>) -> MizeId {
        let mut vec: Vec<String> = Vec::new();
        for i in self {
            vec.push((*i).to_owned())
        }
        instance.id_from_vec_string(vec)
    }
}
impl<S: Store> IntoMizeId<S> for MizeId {
    fn to_mize_id(self, instance: &Instance<S>) -> MizeId {
        self
    }
}

// would only work with a global id_pool
// for now use Instance::new_id()
//impl<T: Into<String>> From<T> for MizeId {
    //fn from(value: T) -> Self {
        //let string = value.into();
        //let path: Vec<String> = string.split("/").map(|v| v.to_owned()).collect();
        //return MizeId { MIZE_ID_POOL.get(path) }
    //}
//}


