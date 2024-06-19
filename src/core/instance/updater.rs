use crossbeam::channel::Receiver;
use tracing::{trace, error};

use crate::{instance::Instance, item::ItemData};
use crate::id::MizeId;
use crate:: error::{MizeResult, MizeError, MizeResultTrait};


#[derive(Debug)]
pub enum Operation {
    Set(MizeId, ItemData),
}

pub fn updater_thread(operation_rx : Receiver<Operation>, mut instance: Instance) -> MizeResult<()> {
    for mut operation in operation_rx {
        trace!("OPERATION");
        let result = handle_operation(&mut operation, &mut instance);

        if let Err(err) = result {
            error!("OPERATION FAILED: {:?}", operation);
        }
    }
    Ok(() )
}

fn handle_operation(operation: &mut Operation, instance: &mut Instance) -> MizeResult<()> {
    match operation {
        Operation::Set(id, value) => {
            let item_data: ItemData = value.to_owned();
            let mut item = instance.get(id.clone())?;
            item.merge(item_data)?;
        }
    }
    Ok(())
}
