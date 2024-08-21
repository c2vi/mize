use core::fmt;
use std::rc::Rc;
use crossbeam::channel::Sender;
use std::sync::Arc;
use tracing::trace;

use crate::id::MizeId;
use crate::instance::connection::Connection;
use crate::item::Item;
use crate::error::MizeResult;
use crate::proto::MizeMessage;

use super::Instance;

#[derive(Clone, Debug)]
pub struct Update {
    pub instance: Arc<Instance>,
    pub id: MizeId,
}

impl Update {
    pub fn new_item(&self) -> MizeResult<Item> {
        self.instance.get(self.id.clone())
    }
}

pub enum Subscription {
    Connection(Connection),
    Closure(Box<dyn Fn(Update) -> MizeResult<()> + Send>),
    Channel(Sender<Update>),
}

impl Subscription {
    pub  fn from_conn(conn: Connection) -> Subscription {
        Subscription::Connection(conn)
    }
    pub fn from_closure(closure: Box<dyn Fn(Update) -> MizeResult<()> + Send>) -> Subscription {
        Subscription::Closure(closure)
    }
    pub fn from_sender(tx: Sender<Update>) -> Subscription {
        Subscription::Channel(tx)
    }

    pub fn handle(&mut self, update: Update) -> MizeResult<()> {
        trace!("handleing update");
        match &self {
            Subscription::Connection(conn) => {
                let msg = MizeMessage::new_update(update.id.clone(), update.new_item()?.as_data_full()?, conn.id);
                conn.send(msg)?;
            }
            Subscription::Closure(closure) => {
                closure(update)?
            }
            Subscription::Channel(tx) => {
                tx.send(update);
            }
        }
        Ok(())
    }
}

impl fmt::Debug for Subscription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "...Subscription...")
    }
}
