use crossbeam::channel::Receiver;
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
    Set(MizeId, ItemData),
    Msg(MizeMessage),
}

pub fn updater_thread(operation_rx : Receiver<Operation>, instance: &Instance) -> MizeResult<()> {
    let mut count = 0;

    loop {
        let mut operation = operation_rx.recv()?;
        let op_str = match operation {
            Operation::Set(_, _) => "SET",
            Operation::Msg(_) => "MSG",
        };

        trace!("OPERATION {} - {}", count, op_str);

        //let mut busy = instance.update_thread_busy.lock()?;

        let result = handle_operation(&mut operation, instance);

        trace!("OPERATION {} DONE", count);
        count += 1;

        if let Err(err) = result {
            error!("OPERATION FAILED: {:?}", operation);
            err.log();
        }

        //drop(busy);
    }
    Ok(())
}

pub fn handle_operation(operation: &mut Operation, instance: &Instance) -> MizeResult<()> {
    match operation {
        Operation::Set(id, value) => {
            let item_data: ItemData = value.to_owned();
            let mut item = instance.get(id.clone())?;
            item.merge(item_data)?;

            //check subs and handle them
            let mut subs_inner = instance.subs.lock()?;
            println!("goooooooooooooooooot some subs: {:?}", subs_inner);
            println!("some subs id: {:?}", id);
            if let Some(vec) = subs_inner.get_mut(&id) {
                let update = Update {
                    instance: Arc::new(instance.to_owned()),
                    id: id.clone(),
                };
                for sub in vec.iter_mut() {
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
            instance.set(id.clone(), data);
        },

        // this should check, if the update is valid
        // but for now, we do just always accept it
        MessageCmd::UpdateRequest => {
            let data = msg.data()?;
            let id = msg.id(instance)?;
            instance.set(id.clone(), data);
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



