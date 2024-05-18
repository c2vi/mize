
impl Connection {
    pub fn from_unix(unix_sock: UnixStream) -> Connection {

    let (recieve_tx, recieve_rx) = tokio::sync::mpsc::channel(crate::instance::UNIX_SOCKET_CHANNEL_SIZE);
    let (send_tx, send_rx) = tokio::sync::mpsc::channel(crate::instance::UNIX_SOCKET_CHANNEL_SIZE);
    let (unix_read, unix_write) = unix_sock.into_split();

    // TODO: error handling and close con on error
    tokio::spawn(unix_send(unix_write, send_rx));
    tokio::spawn(unix_recieve(unix_read, recieve_tx));

    return Connection { tx: send_tx, rx: recieve_rx }

    }
}

async fn unix_send(unix_write: OwnedWriteHalf, send_rx: Receiver<MizeMessage>) -> MizeResult<()> {
    loop {
        let ready = unix_write.ready(Interest::WRITABLE).await.mize_result_msg("hi")?;
        if ready.is_writable() {
            // Try to write data, this may still fail with `WouldBlock`
            // if the readiness event is a false positive.
            match unix_write.try_write(b"hello world") {
                Ok(n) => {
                    println!("wrote {} bytes to unix sock", n);
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }
    }
}

async fn unix_recieve(unix_read: OwnedReadHalf, recieve_tx: Sender<MizeMessage>) -> MizeResult<()> {
    loop {
        let ready = unix_read.ready(Interest::READABLE).await.mize_result_msg("hi")?;

        if ready.is_readable() {
            let mut data = vec![0; 1024];
            // Try to read data, this may still fail with `WouldBlock`
            // if the readiness event is a false positive.
            match unix_read.try_read(&mut data) {
                Ok(n) => {
                    println!("read {} bytes from unix sock", n);        
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }

        }
    }
}
