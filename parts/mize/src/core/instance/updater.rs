use flume::Receiver;
use tracing::{error, trace, warn};
use std::borrow::BorrowMut;
use std::sync::Arc;

use crate::mize_err;
use crate::{instance::Instance, item::ItemData};
use crate::id::MizeId;
use crate:: error::{MizeResult, MizeError, MizeResultTrait};
use crate::proto::{MessageCmd, MizeMessage};

use super::connection::{self, Connection};
use super::subscription::{Subscription, Update};


#[derive(Debug)]
pub enum Operation {
    Set(MizeId, ItemData, Option<Connection>), // bool: is_from_update_msg
    Msg(MizeMessage),
}





// console_log macro
// that can be copied into other files for debugging purposes
#[cfg(feature = "wasm-target")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm-target")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[cfg(feature = "wasm-target")]
macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (unsafe { log(&format_args!($($t)*).to_string())})
}

#[cfg(not(feature = "wasm-target"))]
macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => ()
}
//end of console_log macro





pub async fn updater_thread_async(operation_rx : Receiver<Operation>, instance: Instance) -> () {
    let mut count = 0;
    console_log!("inside an updater thread");

    loop {
        let mut operation = match operation_rx.recv_async().await {
            Ok(val) => val,
            Err(e) => {
                instance.report_err(e.into());
                continue;
            }
        };

        let op_str = match operation {
            Operation::Set(_, _, _) => "SET",
            Operation::Msg(_) => "MSG",
        };

        trace!("OPERATION {} - {}", count, op_str);
        console_log!("OPERATION {} - {}", count, op_str);

        let result = handle_operation(&mut operation, &instance);

        trace!("OPERATION {} DONE", count);
        count += 1;

        if let Err(err) = result {
            error!("OPERATION {} FAILED: {:?}", count, operation);
            err.log();
        }

    }
}

pub fn updater_thread(operation_rx : Receiver<Operation>, instance: &Instance) -> MizeResult<()> {
    let mut count = 0;

    loop {
        let mut operation = operation_rx.recv()?;
        let op_str = match operation {
            Operation::Set(_, _, _) => "SET",
            Operation::Msg(_) => "MSG",
        };

        trace!("OPERATION {} - {}", count, op_str);

        let result = handle_operation(&mut operation, instance);

        trace!("OPERATION {} DONE", count);
        count += 1;

        if let Err(err) = result {
            error!("OPERATION FAILED: {:?}", operation);
            err.log();
        }

    }
    Ok(())
}

pub fn handle_operation(operation: &mut Operation, instance: &Instance) -> MizeResult<()> {

    match operation {
        Operation::Set(id, value, maybe_conn) => {
            let item_data: ItemData = value.to_owned();
            let mut item = instance.get(id.clone())?;
            item.merge(item_data)?;

            //check subs and handle them
            let mut subs_inner = instance.subs.lock()?;
            if let Some(vec) = subs_inner.get_mut(&id) {
                let update = Update {
                    instance: Arc::new(instance.to_owned()),
                    id: id.clone(),
                };
                for sub in vec.iter_mut() {

                    // don't handle sub of type connection, in case the update comes from this
                    // connection
                    if let Some(conn) = maybe_conn {
                        if let Subscription::Connection(conn2) = sub {
                            if conn.id == conn2.id {
                                continue;
                            }
                        }
                    }

                    sub.handle(update.clone());
                }
            }
        },
        Operation::Msg(msg) => {
            handle_msg(msg, instance)?
        },
    }
    Ok(())
}

fn handle_msg(msg: &mut MizeMessage, instance: &Instance) -> MizeResult<()> {
    match msg.cmd()? {
        MessageCmd::Get => {
            let id = msg.id(instance)?;
            let mut connection = instance.get_connection(msg.conn_id)?.clone();
            let item = instance.get(id.clone())?;
            let msg = MizeMessage::new_give(id, item.as_data_full()?, msg.conn_id);
            connection.send(msg)?;
        },

        MessageCmd::GetSub => {
            let id = msg.id(instance)?;
            let mut connection = instance.get_connection(msg.conn_id)?.clone();
            let item = instance.get(id.clone())?;
            let msg = MizeMessage::new_give(id.clone(), item.as_data_full()?, msg.conn_id);
            connection.send(msg)?;
            let sub = Subscription::from_conn(connection.clone());
            instance.sub(id, sub)?;
        },

        MessageCmd::Sub => {
            let id = msg.id(instance)?;
            let mut connection = instance.get_connection(msg.conn_id)?.clone();
            let sub = Subscription::from_conn(connection.clone());
            instance.sub(id, sub)?;
        },

        MessageCmd::Update => {
            let data = msg.data()?;
            let id = msg.id(instance)?;
            let connection = instance.get_connection(msg.conn_id)?;
            instance.op_tx.send(Operation::Set(id.clone(), data, Some(connection)));
        },

        // this should check, if the update is valid
        // but for now, we do just always accept it
        MessageCmd::UpdateRequest => {
            let data = msg.data()?;
            let id = msg.id(instance)?;
            let connection = instance.get_connection(msg.conn_id)?;
            instance.op_tx.send(Operation::Set(id.clone(), data, Some(connection)));
        },

        MessageCmd::Give => {
            let id = msg.id(instance)?;
            let data = msg.data()?;
            let give_msg_wait_inner = instance.give_msg_wait.lock()?;
            if let Some(vec) = give_msg_wait_inner.get(&id) {
                for tx in vec {
                    tx.send(data.clone());
                }
            } else {
                warn!("got give msg for id '{}', that has no one waiting for it", id);
            }
        },

        MessageCmd::Create => {
            println!("instance.store: {:?}", instance.clone().store);
            let item = instance.new_item()?;
            let reply_msg = MizeMessage::new_create_reply(item.id(), msg.conn_id);
            let mut connection = instance.get_connection(msg.conn_id)?;
            connection.send(reply_msg)?;
        }

        MessageCmd::CreateReply => {
            let create_msg_wait_inner = instance.create_msg_wait.lock()?;
            if let Some(sender) = create_msg_wait_inner.as_ref() {
                sender.send(msg.id(&mut instance.clone())?)?;
            }
            return Ok(());
        }
        _ => {
            return Err(mize_err!("got a message, that is not handeled"));
        },
    }
    Ok(())
}



