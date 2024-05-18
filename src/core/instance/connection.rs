use log::{trace, debug, info, warn, error};
use tokio::sync::mpsc::{Sender, Receiver};

use crate::proto::MizeMessage;
use crate::error::{MizeError, MizeResult, IntoMizeResult};

pub trait Connection {
}


//#[derive(Debug)]
//pub struct Connection {
    //tx: Sender<MizeMessage>,
    //rx: Receiver<MizeMessage>,
//}

