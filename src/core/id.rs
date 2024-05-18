use std::hash::Hash;


#[derive(Hash, Eq, PartialEq)]
pub struct MizeId {
    path: Vec<String>
}

impl From<String> for MizeId {
    fn from(value: String) -> Self {
        MizeId{path: vec![value]}
    }
}
