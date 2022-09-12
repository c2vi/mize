
use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use hyper::body::Bytes;

#[tokio::main]
async fn hyper_server() {
    // Construct our SocketAddr to listen on...
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // And a MakeService to handle each connection...
    let make_service = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(handle))
    });

    // Then bind and serve...
    let server = Server::bind(&addr).serve(make_service);
    
    // And run forever...
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn handle(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let (mut sender, _body) = Body::channel();
    sender.send_data(Bytes::from("sending from the sender\n"));
    sender.abort();
    println!("{:?}", req.body());
    //println!("{:?}", Body::to_bytes());
    Ok(Response::new(Body::from("hi")))
}
