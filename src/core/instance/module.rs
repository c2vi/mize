

// https://users.rust-lang.org/t/casting-between-trait-object-types/97220/2

use crate::instance::Instance;
use crate::error::MizeResult;

pub trait Module {
    fn init(&mut self, instance: &Instance) -> MizeResult<()>;

    fn exit(&mut self, instance: &Instance) -> MizeResult<()>;
}


pub struct EmptyModule {
}

impl Module for EmptyModule {
    fn init(&mut self, instance: Instance) -> MizeResult<()> {
        println!("empty module fn init");
        Ok(())
    }
    fn exit(&mut self, instance: Instance) -> MizeResult<()> {
        println!("empty module fn exit");
        Ok(())
        
    }
}

