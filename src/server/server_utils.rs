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
    fields: Vec<Fields>
}

pub fn import(args: Vec<String>){

}

pub fn update_item(id: u64, key: String, update_string: String){

}

pub fn read_index(id: u64, mize_folder: &str) -> io::Result<Index>{
    let mut index_folder_num: String = String::new();
    let mut index_num: String = String::new();
    let id_len = id.to_string().chars().count();

    if id_len <= 6 {
        index_folder_num = String::from("0");
        index_num = id.to_string();
    } else {
        index_folder_num = String::from(&id.to_string()[..id_len-6]);
        index_num = String::from(&id.to_string()[id_len-6..]);
    }

    let index_file_path = String::from(mize_folder) + "/index" + &index_folder_num + "/" + &index_num;

    let mut file = File::open(index_file_path)?;

    let mut commit_number_buff: [u8; 8];
    file.read_exact(&mut commit_number_buff[..])?;

    let fields: Vec<Fields> = Vec::new();

    Ok(Index {
        commit_number: u64::from_le_bytes(commit_number_buff),

    })

}

pub fn init_mize_folder(mize_folder: String) -> Result<(), MizeError>{
    //##check if the folder resembles a mize folder

    //## check if the index0 file exists and create one if it does not
    let index_path = mize_folder.clone() + "/index0";
    if !Path::new(&index_path).exists() {
        File::create(&index_path).unwrap();
    }
    let index_md = fs::metadata(&index_path).unwrap();
    if index_md.is_dir(){
        return Err(MizeError{message: String::from("index is a folder and not a file")})
    }

    let folders: [&str; 5] = ["fields0","keys0","data0","config","other"];
    for folder in folders {
        //## check if the folder exists and create one if it does not
        let path = mize_folder.clone() + "/" + folder;
        if !Path::new(&path).exists() {
            fs::create_dir(&path).unwrap();
        }
        let index_md = fs::metadata(&path).unwrap();
        if index_md.is_file(){
            return Err(MizeError{message: String::from(folder) + " is a file and not a folder"});
        }
    }

    let index = read_index(0, &mize_folder);

    //if let Ok(i) = index {
        
    //}
    if let Err(i) = index{
        println!("{:?}", i.kind());
        match i.kind() {
            io::ErrorKind::UnexpectedEof => {
                //create the first item
                
                //create index for first item
                File::create(mize_folder.clone() + "/keys0/0");

                //create keyfile in keyfile0
                File::create(mize_folder.clone() + "/keys0/000_000");
            },
            _ => return Err(MizeError{
                message: String::from("sth went wrong when reading the index of item 0")
            })
        }
    }
    Ok(())
}


