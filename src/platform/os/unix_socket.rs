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
use crate::proto::{self, MizeMessage};

pub struct UnixListener {
    sock_path: PathBuf,
}

impl UnixListener {
    pub fn new(store_path: PathBuf) -> MizeResult<UnixListener> {
        Ok(UnixListener { sock_path: store_path.join("sock").to_owned() })
    }

}


pub fn connect(instance: &mut Instance, store_path: PathBuf) -> MizeResult<()> {
    let conn_id = instance.spawn_async_blocking("unix_connection", connect_async(instance.clone(), store_path) )?;

    let ns_of_peer_str = instance.get(format!("inst/con_by_id/{}/peer/0/config/namespace", conn_id))?.value_string()?;
    let ns_of_peer = instance.namespace_from_string(ns_of_peer_str)?;

    info!("connect_async ... ns_of_peer: {}", ns_of_peer.clone().as_string());

    instance.connection_set_namespace(conn_id, ns_of_peer.clone());
    instance.set_namespace(ns_of_peer);

    Ok(())
}


async fn connect_async(mut instance: Instance, store_path: PathBuf) -> MizeResult<u64> {
    let mut stream = UnixStream::connect(store_path.join("sock")).await?;
    let (unix_read, unix_write) = stream.into_split();

    let (send_tx, send_rx) = unbounded::<MizeMessage>();
    
    let conn_id = instance.new_connection(send_tx)?;


    let cloned_instance = instance.clone();
    instance.spawn("incomming", move || {
        let result = unix_incomming(unix_read, cloned_instance, conn_id);
        // if unix incomming fails, close the connection
        if let Err(err) = result {
            warn!("Connection closing because of EOF");
        }
        Ok(())
    });

    let outgoing_cloned_instance = instance.clone();
    instance.spawn("outgoing", move || {
        let result = unix_outgoing(unix_write, send_rx, outgoing_cloned_instance, conn_id);
        // if writing fails, close this connection
        if let Err(err) = result {
            warn!("Connection closing because of EOF");
        }
        Ok(())
    });

    return Ok(conn_id);
}


impl ConnListener for UnixListener {
    fn listen(self, mut instance: Instance) -> MizeResult<()> {
        instance.spawn_async("unix listen async", unix_listen(self, instance.clone()));
        Ok(())
    }
}


async fn unix_listen(listener: UnixListener, mut instance: Instance) -> MizeResult<()> {
    // remove the file at sock_path if it already exists
    fs::remove_file(&listener.sock_path);

    let listener = TokioUnixListener::bind(&listener.sock_path)
        .mize_result_msg(format!("Could not bind to unix socket at '{}'", listener.sock_path.display()))?;

    loop {
        let (mut unix_sock, addr) = listener.accept().await
            .mize_result_msg("Error while accepting Unix sock connection")?;
        info!("new connection");

        let (send_tx, send_rx) = unbounded::<MizeMessage>();
        let (unix_read, unix_write) = unix_sock.into_split();


        let conn_id = instance.new_connection(send_tx)?;
        let cloned_instance = instance.clone();
        instance.spawn("incomming", move || {
            let result = unix_incomming(unix_read, cloned_instance, conn_id);
            // if unix incomming fails, close the connection
            if let Err(err) = result {
                warn!("Connection closing because of EOF");
            }
            Ok(())
        });

        let outgoing_cloned_instance = instance.clone();
        instance.spawn("outgoing", move || {
            let result = unix_outgoing(unix_write, send_rx, outgoing_cloned_instance, conn_id);
            // if writing fails, close this connection
            if let Err(err) = result {
                warn!("Connection closing because of EOF");
            }
            Ok(())
        });
    }
}


fn unix_outgoing(mut unix_write: OwnedWriteHalf, send_rx: Receiver<MizeMessage>, mut instance: Instance, conn_id: u64) -> MizeResult<()> {
    for msg in send_rx {
        println!("unix outgoing got msg: {}", msg);
        let adapter = MyCiboriumWriter { inner: &mut unix_write, instance: &mut instance };
        let value = msg.value();
        ciborium::into_writer(&value, adapter)?
    }
    Ok(())
}

fn unix_incomming(mut unix_read: OwnedReadHalf, mut instance: Instance, conn_id: u64) -> MizeResult<()> {
    loop {
        //println!("unix incoming loop...");
        let adapter = MyCiboriumReader { inner: &mut unix_read, instance: &mut instance };
        let value: CborValue = ciborium::from_reader(adapter)?;
        if let CborValue::Integer(_) = value {
            break;
        }
        let msg = MizeMessage::new(value, conn_id);
        println!("unix incoming got msg: {}", msg);
        println!("op channel len: {}", instance.op_tx.len());
        instance.got_msg(msg);
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
        //println!("CALL: write_all");
        let runtime = self.instance.runtime.lock()?;
        //println!("after runtime");
        let num_read = runtime.block_on(self.inner.write_all(data))?;
        Ok(())
    }
    fn flush(&mut self) -> Result<(), Self::Error> {
        //println!("CALL flush");
        let handle = self.instance.async_get_handle();
        let result = handle.block_on(self.inner.flush());
        //println!("flush result: {:?}", result);
        Ok(())
    }
}

struct MyCiboriumReader<'a> {
    inner: &'a mut OwnedReadHalf,
    instance: &'a mut Instance,
}

async fn test(test: &mut OwnedReadHalf, buf: &mut [u8]) {
    //println!("hello from async...");
    //println!("len: {}", buf.len());
    let num_read = test.read_u8().await;
    //println!("num_read: {:?}", num_read);
    //test.read_exact(buf).await;
    //println!("hello from async... twoooooooooooooo");
}

impl<'a> ciborium_io::Read for MyCiboriumReader<'a> {
    type Error = MizeError;
    fn read_exact(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
        //println!("data before: {:?}", data);
        //let runtime = self.instance.runtime.lock()?;
        //println!("are we here after runtime lock()");
        //runtime.block_on(test(self.inner, data));
        //println!("are we hereeeeeeeeeeeeeeeeeeee");

        //return Ok(());

        //println!("incoming before block_on");
        let handle = self.instance.async_get_handle();
        let num_read = handle.block_on(self.inner.read_exact(data))?;
        //println!("incoming num_read: {:?}", num_read);
        //println!("data after: {:?}", data);

        // if we read 0 bytes, that means the reader stream closed and we should terminate the
        // connection
        //if num_read == 0 {
            //return Err(MizeError::new().msg("ReceiverStream closed"));
        //}
        Ok(())
    }
}

