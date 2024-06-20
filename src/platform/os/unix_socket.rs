use std::fs;
use std::path::{Path, PathBuf};
use tokio::net::UnixListener as TokioUnixListener;
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
use tokio::io::{Interest, AsyncReadExt};
use crossbeam::channel::{Receiver, Sender, unbounded};
use ciborium::Value as CborValue;

use crate::instance::peer::{PeerListener, Peer};
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

impl PeerListener for UnixListener {
    fn listen(self, mut instance: Instance) -> MizeResult<()> {
        instance.spawn_async("unix listen async", unix_listen(self, instance.clone()));
        Ok(())
    }
}


async fn unix_listen(listener: UnixListener, mut instance: Instance) -> MizeResult<()> {
    fs::remove_file(&listener.sock_path)?;
    let listener = TokioUnixListener::bind(&listener.sock_path)
        .mize_result_msg(format!("Could not bind to unix socket at '{}'", listener.sock_path.display()))?;

    loop {
        let (mut unix_sock, addr) = listener.accept().await
            .mize_result_msg("Error while accepting Unix sock connection")?;

        let (recieve_tx, recieve_rx) = unbounded::<MizeMessage>();
        let (send_tx, send_rx) = unbounded::<MizeMessage>();
        let (unix_read, unix_write) = unix_sock.into_split();


        let peer = Peer::new(recieve_rx, send_tx);

        instance.add_peer(peer);
        let cloned_instance = instance.clone();
        instance.spawn("incomming", || unix_incomming(unix_read, recieve_tx, cloned_instance));
        instance.spawn_async("outgoing", unix_outgoing(unix_write, send_rx));
    }
}


async fn unix_outgoing(unix_write: OwnedWriteHalf, send_rx: Receiver<MizeMessage>) -> MizeResult<()> {
    return Ok(());
    loop {
        let ready = unix_write.ready(Interest::WRITABLE).await.mize_result_msg("hi")?;
        if ready.is_writable() {
            // Try to write data, this may still fail with `WouldBlock`
            // if the readiness event is a false positive.
            match unix_write.try_write(b"hello world") {
                Ok(n) => {
                    println!("wrote {} bytes to unix sock", n);
                }
                Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }
    }
    Ok(())
}

struct MyAdapter<'a> {
    inner: &'a mut OwnedReadHalf,
    instance: &'a mut Instance,
}

impl<'a> ciborium_io::Read for MyAdapter<'a> {
    type Error = MizeError;
    fn read_exact(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
        let runtime = self.instance.runtime.lock()?;
        let num_read = runtime.block_on(self.inner.read_exact(data))?;
        println!("read {} bytes", num_read);
        Ok(())
    }
}

fn unix_incomming(mut unix_read: OwnedReadHalf, recieve_tx: Sender<MizeMessage>, mut instance: Instance) -> MizeResult<()> {
    loop {
        let adapter = MyAdapter { inner: &mut unix_read, instance: &mut instance };
        let value: CborValue = ciborium::from_reader(adapter)?;
        if let CborValue::Integer(_) = value {
            break;
        }
        println!("value: {:?}", value);
    }
    Ok(())
}

/*


    let mut buf: Vec<u8> = vec![2,2,2];

    let mut count: usize = 0;

    loop {
        let num_bytes = unix_read.read(&mut buf[..]).await?;
        let string = String::from_utf8(buf.clone())?;
        println!("read num: {}", num_bytes);
        println!("incoming: {}", string);
        if num_bytes == 0 {
            println!("zerooooooooooooooooooo");
            break
        }
        count += 1;
        if count > 20 {
            break
        }
    }
    println!("end of loop");

    return Ok(());
    let mut count: usize = 0;
    loop {
        let num_bytes = unix_read.read(&mut buf[..]).await?;
        let string = String::from_utf8(buf.clone())?;
        println!("read num: {}", num_bytes);
        println!("incoming: {}", string);
        //if num_bytes == 0 {
            //break
        //}
        count += 1;
        if count > 20 {
            break
        }
    }
    return Ok(());

    loop {
        let ready = unix_read.ready(Interest::READABLE).await.mize_result_msg("unix_read_half.ready() failed")?;

        if ready.is_readable() {
            let mut data = vec![0; 1024];
            // Try to read data, this may still fail with `WouldBlock`
            // if the readiness event is a false positive.
            match unix_read.try_read(&mut data) {
                Ok(n) => {
                    println!("read {} bytes from unix sock", n);
                }
                Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }

        }
    }
}
// */
