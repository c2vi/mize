use crossbeam::channel::{Sender, Receiver};
use tracing::{trace, debug, info, warn, error};

use crate::proto::MizeMessage;
use crate::error::{MizeError, MizeResult, IntoMizeResult};

use super::Instance;

#[derive(Clone)]
pub struct Connection {
    pub rx: Receiver<MizeMessage>,
    pub tx: Sender<MizeMessage>,
    pub id: u64,
}

pub trait ConnListener : Send + Sync {
    fn listen(self, instance: Instance) -> MizeResult<()>;
}

impl Connection {
    pub fn send(&mut self, msg: MizeMessage) -> MizeResult<()> {
        Ok(self.tx.send(msg)?)
    }
}



