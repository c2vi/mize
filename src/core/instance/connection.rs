use flume::{Sender, Receiver};
use tracing::{trace, debug, info, warn, error};

use crate::id::Namespace;
use crate::mize_err;
use crate::proto::MizeMessage;
use crate::error::{MizeError, MizeResult, IntoMizeResult};

use super::Instance;
use crate::item::{get_raw_from_cbor, Item, ItemData};

// only created with Instance::new_connection()
#[derive(Clone, Debug)]
pub struct Connection {
    pub tx: Sender<MizeMessage>,
    pub id: u64,
    pub ns: Option<Namespace>,
}

pub trait ConnListener : Send + Sync {
    fn listen(self, instance: Instance) -> MizeResult<()>;
}

impl Connection {
    pub fn send(&self, msg: MizeMessage) -> MizeResult<()> {
        Ok(self.tx.send(msg)?)
    }
}

pub fn value_raw_con_by_id(item: &mut Item) -> MizeResult<ItemData> {
    if item.id().nth_part(3)? == "peer" {
        return value_raw_from_peer(item);
    } else {
        return Err(mize_err!("id with path: '{}' is not supported", item.id()));
    }
}

pub fn value_raw_from_peer(item: &mut Item) -> MizeResult<ItemData> {
    let id = item.id();

    let conn_id_str = id.nth_part(2)?;
    let conn_id : u64 = conn_id_str.parse()?;
    let tmp_path = id.path();
    let mut new_id_str = tmp_path.into_iter();
    let skipped = new_id_str.skip(4);
    let new_id = item.instance.new_id(skipped.collect::<Vec<&String>>())?;

    let connection = item.instance.get_connection(conn_id)?;

    let msg = MizeMessage::new_get(new_id.clone(), connection.id);

    connection.tx.send(msg)?;

    let data = item.instance.give_msg_wait(new_id)?;

    return Ok(data);
}


