use crossbeam::channel::{Sender, Receiver};
use tracing::{trace, debug, info, warn, error};

use crate::proto::MizeMessage;
use crate::error::{MizeError, MizeResult, IntoMizeResult};

use super::Instance;

pub struct Peer {
    rx: Receiver<MizeMessage>,
    tx: Sender<MizeMessage>,
}

pub trait PeerListener : Send + Sync {
    fn listen(self, instance: Instance) -> MizeResult<()>;
}

impl Peer {
    pub fn new(rx: Receiver<MizeMessage>, tx: Sender<MizeMessage>) -> Peer {
        Peer { rx , tx }
    }
}



