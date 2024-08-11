use std::fs;
use std::path::{Path, PathBuf};
use tokio::net::{UnixListener as TokioUnixListener, UnixStream};
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
use tokio::io::{Interest, AsyncReadExt, AsyncWriteExt};
use crossbeam::channel::{Receiver, Sender, unbounded};
use ciborium::Value as CborValue;
use tracing::{info, warn};

use crate::instance::connection::{ConnListener, Connection};
use crate::instance::{self, Instance};
use crate::error::{MizeError, MizeResult, IntoMizeResult};
use crate::proto::MizeMessage;

pub struct UnixListener {
    sock_path: PathBuf,
}

impl UnixListener {
    pub fn new(store_path: PathBuf) -> MizeResult<UnixListener> {
        Ok(UnixListener { sock_path: store_path.join("sock").to_owned() })
    }

}


pub fn connect(instance: &mut Instance, store_path: PathBuf) -> MizeResult<()> {
    instance.spawn_async("unix_connection", connect_async(instance.clone(), store_path) );

    Ok(())
}


async fn connect_async(mut instance: Instance, store_path: PathBuf) -> MizeResult<()> {
    let mut stream = UnixStream::connect(store_path).await?;
    let (read_half, write_half) = stream.split();

    let (recieve_tx, recieve_rx) = unbounded::<MizeMessage>();
    let (send_tx, send_rx) = unbounded::<MizeMessage>();
    
    let conn_id = instance.new_connection(recieve_rx, send_tx)?;
    let ns_of_peer = instance.get(format!("inst/con_by_id/{}/peer/0/config/namespace", conn_id))?.value_string()?;

    info!("connect_async ... ns_of_peer: {}", ns_of_peer);

    //instance.connection_set_namespace(conn_id, namespace)

    Ok(())
}


impl ConnListener for UnixListener {
    fn listen(self, mut instance: Instance) -> MizeResult<()> {
        instance.spawn_async("unix listen async", unix_listen(self, instance.clone()));
        Ok(())
    }
}


async fn unix_listen(listener: UnixListener, mut instance: Instance) -> MizeResult<()> {
    // remove the file at sock_path if it already exists
    fs::remove_file(&listener.sock_path)?;

    let listener = TokioUnixListener::bind(&listener.sock_path)
        .mize_result_msg(format!("Could not bind to unix socket at '{}'", listener.sock_path.display()))?;

    loop {
        let (mut unix_sock, addr) = listener.accept().await
            .mize_result_msg("Error while accepting Unix sock connection")?;

        let (recieve_tx, recieve_rx) = unbounded::<MizeMessage>();
        let (send_tx, send_rx) = unbounded::<MizeMessage>();
        let (unix_read, unix_write) = unix_sock.into_split();


        let conn_id = instance.new_connection(recieve_rx, send_tx)?;
        let cloned_instance = instance.clone();
        instance.spawn("incomming", move || {
            let result = unix_incomming(unix_read, recieve_tx, cloned_instance, conn_id);
            // if unix incomming fails, close the connection
            if let Err(err) = result {
                warn!("Connection closing")
            }
            Ok(())
        });

        let outgoing_cloned_instance = instance.clone();
        instance.spawn("outgoing", move || {
            let result = unix_outgoing(unix_write, send_rx, outgoing_cloned_instance, conn_id);
            // if writing fails, close this connection
            if let Err(err) = result {
                warn!("Connection closing")
            }
            Ok(())
        });
    }
}


fn unix_outgoing(mut unix_write: OwnedWriteHalf, send_rx: Receiver<MizeMessage>, mut instance: Instance, conn_id: u64) -> MizeResult<()> {
    for msg in send_rx {
        let adapter = MyCiboriumWriter { inner: &mut unix_write, instance: &mut instance };
        let value = msg.value();
        ciborium::into_writer(&value, adapter)?
    }
    Ok(())
}

fn unix_incomming(mut unix_read: OwnedReadHalf, recieve_tx: Sender<MizeMessage>, mut instance: Instance, conn_id: u64) -> MizeResult<()> {
    loop {
        let adapter = MyCiboriumReader { inner: &mut unix_read, instance: &mut instance };
        let value: CborValue = ciborium::from_reader(adapter)?;
        if let CborValue::Integer(_) = value {
            break;
        }
        let msg = MizeMessage::new(value, conn_id);
        recieve_tx.send(msg)?;
    }
    Ok(())
}


struct MyCiboriumWriter<'a> {
    inner: &'a mut OwnedWriteHalf,
    instance: &'a mut Instance,
}

impl<'a> ciborium_io::Write for MyCiboriumWriter<'a> {
    type Error = MizeError;
    fn write_all(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        let runtime = self.instance.runtime.lock()?;
        let num_read = runtime.block_on(self.inner.write_all(data))?;
        Ok(())
    }
    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

struct MyCiboriumReader<'a> {
    inner: &'a mut OwnedReadHalf,
    instance: &'a mut Instance,
}

impl<'a> ciborium_io::Read for MyCiboriumReader<'a> {
    type Error = MizeError;
    fn read_exact(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
        let runtime = self.instance.runtime.lock()?;
        let num_read = runtime.block_on(self.inner.read_exact(data))?;

        // if we read 0 bytes, that means the reader stream closed and we should terminate the
        // connection
        if num_read == 0 {
            return Err(MizeError::new().msg("ReceiverStream closed"));
        }
        Ok(())
    }
}

