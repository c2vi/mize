use core::fmt;
use std::{collections::binary_heap::Iter, fmt::write, hash::Hash};
use interner::{shared::{SharedPool, SharedString, SharedVecString}, Pooled};
use std::collections::hash_map::RandomState;
use std::path::Path;

use crate::{instance::{store::Store, Instance}, mize_err};
use crate::error::{MizeResult, MizeError, MizeResultTrait};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct MizeId {
    pub path: SharedVecString,
    pub namespace: Namespace,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Namespace ( pub SharedString );

impl Namespace {
    pub fn as_string(&self) -> SharedString {
        self.0.clone()
    }
    pub fn as_real_string(&self) -> String {
        let a =  self.as_string();
        let b = a.to_string();
        return b;
    }
}

pub trait IntoMizeId {
    fn to_mize_id(self, instance: &Instance) -> MizeResult<MizeId>;
}


impl MizeId {
    pub fn store_part(&self) -> &str {
        self.path.iter().nth(0)
            .expect("an empty MizeId found, that should absolutely not be possible!!!!").as_str()
    }

    pub fn nth_part(&self, n: usize) -> MizeResult<&str> {
        match self.path.iter().nth(n) {
            Some(val) => Ok(val.as_str()),
            None => Err(mize_err!("id does not have {} parts", n)),
        }
    }

    pub fn path(&self) -> SharedVecString {
        self.path.clone()
    }

    pub fn after_store_part(&self) -> Vec<String> {
        let tmp = self.path();
        let mut path_iter = tmp.into_iter();
        path_iter.next();
        let new_path: Vec<String> = path_iter.map(|v| v.to_owned()).collect();

        return new_path;
    }

    pub fn namespace(&self) -> Namespace {
        self.namespace.clone()
    }

    pub fn namespace_str(&self) -> &str {
        &self.namespace.0
    }
}

impl fmt::Display for MizeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path.join("/"))
    }
}

impl IntoMizeId for &str {
    fn to_mize_id(self, instance: &Instance) -> MizeResult<MizeId> {
        instance.id_from_string(self.to_owned())
    }
}

impl IntoMizeId for String {
    fn to_mize_id(self, instance: &Instance) -> MizeResult<MizeId> {
        instance.id_from_string(self)
    }
}
impl IntoMizeId for &String {
    fn to_mize_id(self, instance: &Instance) -> MizeResult<MizeId> {
        instance.id_from_string(self.to_owned())
    }
}
impl IntoMizeId for Vec<String> {
    fn to_mize_id(self, instance: &Instance) -> MizeResult<MizeId> {
        instance.id_from_vec_string(self)
    }
}
impl IntoMizeId for Vec<&String> {
    fn to_mize_id(self, instance: &Instance) -> MizeResult<MizeId> {
        let owned = self.into_iter().map(|s| s.to_owned()).collect::<Vec<String>>();
        instance.id_from_vec_string(owned)
    }
}
impl IntoMizeId for &[&str] {
    fn to_mize_id(self, instance: &Instance) -> MizeResult<MizeId> {
        let mut vec: Vec<String> = Vec::new();
        for i in self {
            vec.push((*i).to_owned())
        }
        instance.id_from_vec_string(vec)
    }
}
impl IntoMizeId for Vec<&str> {
    fn to_mize_id(self, instance: &Instance) -> MizeResult<MizeId> {
        let mut vec: Vec<String> = Vec::new();
        for i in self {
            vec.push((*i).to_owned())
        }
        instance.id_from_vec_string(vec)
    }
}
impl IntoMizeId for MizeId {
    fn to_mize_id(self, instance: &Instance) -> MizeResult<MizeId> {
        Ok(self)
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


#[cfg(test)]
mod test {
    use crate::instance::Instance;

    use super::*;

    #[test]
    fn test_id() -> MizeResult<()> {
        let instance = Instance::empty()?;

        let id_one = instance.new_id("0/config/hi")?;
        assert_eq!(instance.get_namespace()?, id_one.namespace());
        assert_eq!("0", id_one.store_part());
        assert_eq!(vec!["config".to_owned(), "hi".to_owned()], id_one.after_store_part());

        let id_two = instance.new_id("test.ns:hello/config/hi")?;
        assert_eq!(instance.namespace_from_string("test.ns".to_owned())?, id_two.namespace());
        assert_eq!("hello", id_two.store_part());
        assert_eq!(vec!["config".to_owned(), "hi".to_owned()], id_one.after_store_part());

        let id_three = instance.new_id("0/")?;
        assert_eq!(instance.get_namespace()?, id_three.namespace());
        assert_eq!("0", id_three.store_part());
        assert_eq!(Vec::<String>::new(), id_three.after_store_part());

        let id_four = instance.new_id("test.ns:0/")?;
        assert_eq!(instance.namespace_from_string("test.ns".to_owned())?, id_four.namespace());
        assert_eq!("0", id_four.store_part());
        assert_eq!(Vec::<String>::new(), id_four.after_store_part());

        let id_fife = instance.new_id("0")?;
        assert_eq!(instance.get_namespace()?, id_fife.namespace());
        assert_eq!("0", id_fife.store_part());
        assert_eq!(Vec::<String>::new(), id_fife.after_store_part());

        //TODO: add the parsing form mize:// urls....


        Ok(())
    }


}
