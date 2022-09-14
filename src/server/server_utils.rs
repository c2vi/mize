use std::io::{self, ErrorKind, SeekFrom};
use std::collections::HashMap;
use std::io::prelude::*;
use std::fs;
use std::fs::File;
use std::path::Path;
use crate::error::MizeError;

pub struct Fields{
    key: u64,
    data: u64
}

pub struct Index{
    commit_number: u64,
    //fields: Vec<Fields>
}

pub fn import(args: Vec<String>){

}

pub fn update_item(id: u64, key: String, update_string: String){

}



