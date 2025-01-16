

// https://users.rust-lang.org/t/casting-between-trait-object-types/97220/2

use std::ffi::OsString;

use crate::instance::Instance;
use crate::error::MizeResult;

pub trait Module: {
    fn init(&mut self, instance: &Instance) -> MizeResult<()>;

    fn exit(&mut self, instance: &Instance) -> MizeResult<()>;

    fn clone_module(&self) -> Box<dyn Module + Send + Sync>;


    // extra traits, that can be implemented
    // return None if not implemented

    fn run_cli(&mut self, instance: &Instance, cmd_line: Vec<OsString>) -> Option<MizeResult<()>> {
        None
    }
}


pub struct EmptyModule {
}


impl Module for EmptyModule {
    fn init(&mut self, instance: &Instance) -> MizeResult<()> {
        println!("empty module fn init");
        Ok(())
    }
    fn exit(&mut self, instance: &Instance) -> MizeResult<()> {
        println!("empty module fn exit");
        Ok(())
        
    }

    fn clone_module(&self) -> Box<dyn Module + Send + Sync> {
        Box::new(EmptyModule {  })
    }
}

