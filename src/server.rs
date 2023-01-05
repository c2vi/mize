

pub mod server_utils;
pub mod itemstore;
pub mod proto;

//use futures_util::lock::Mutex;
use futures_util::{FutureExt, StreamExt, SinkExt};
use warp::Filter;
use warp::ws::{WebSocket, self};
use std::collections::HashMap;

use tokio::sync::mpsc::{Sender, channel, Receiver};
use tokio_stream::wrappers::ReceiverStream;
use std::alloc::handle_alloc_error;
use std::fs;
use std::sync::{Arc};
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use crate::server::proto::handle_mize_message;

use self::itemstore::Itemstore;

//static API_PATH_STRING: &str = "$api";
static SOCKET_CHANNEL_SIZE: usize = 200;

// max file sizez in the mize/data folder (in bytes)
static MAX_INDEX_FILE_SIZE: usize = 1_800_000;
static MAX_FIELDS_FILE_SIZE: usize = 2_600_000;
static MAX_KEYS_FILE_SIZE: usize = 30_000_000;
static MAX_DATA_FILE_SIZE: usize = 3_000_000_000;


//static HELP_MESSAGE: &str = "\
//Usage:
    //mize-server server [options]
//
//Available options
    //-h --help           prints this help message
    //-v --version        prints the version
    //--folder=<folder>   mize-folder: where mize stores all it's stuff.
//";

//some flags e.g: "--file /tmp" can require that the next argument belongs to them instead of being
//the command
//static FLAGS_WITH_ARGUMENTS: [&str; 0] = [];

//static AVAILABLE_COMMANDS: [&str; 3] = ["run", "help", "version"];

//static AVAILABLE_FLAGS: [&str; 2] = ["--version", "--help"];
//static AVAILABLE_ONE_LETTER_FLAGS: [&str; 2] = ["v", "h"];

//static VERSION_MESSAGE: &str = "\
//Version: 0
//";


#[derive(Clone, Debug)]
pub struct Client {
//    rx: UnboundedReceiverStream<Result<ws::Message, warp::Error>>,
    tx: Sender<proto::Message>,
    id: u64,
}

#[derive(Clone, Debug)]
pub struct Module {
    tx: Sender<proto::Message>,
    name: String,
    client_id: u64,
    //later
    //registered_types: Vec<String>,
}

//pub struct Upstream maybe???? ... I'd say later....


// A collection of all the things, that most functions need
#[derive(Clone)]
pub struct Mutexes {
    next_free_client_id: Arc<Mutex<u64>>,
    clients: Arc<Mutex<Vec<Client>>>,
    modules: Arc<Mutex<Vec<Module>>>,
    subs: Arc<Mutex<HashMap<String, Vec<proto::Origin>>>>,
    //maybe a list of upstream servers??
    itemstore: Arc<Mutex<Itemstore>>,
}

impl Mutexes {
    pub fn clone(mutexes: &Mutexes) -> Mutexes {
        Mutexes {
            next_free_client_id: Arc::clone(&mutexes.next_free_client_id),
            clients: Arc::clone(&mutexes.clients),
            modules: Arc::clone(&mutexes.modules),
            subs: Arc::clone(&mutexes.subs),
            itemstore: Arc::clone(&mutexes.itemstore),
        }
    }
}

#[tokio::main]
pub async fn run_server(args: Vec<String>) {
    /*
     * WHAT IT DOES
     *
     */

    //
    //### get the mize_folder
    let mut mize_folder = String::new();
    for i in 1..args.len() {
        if args[i].contains('=') {
            let split: Vec<_> = args[i].split('=').collect();
            if split[0] == "--folder" {
                mize_folder += split[1];
            }
        }
    }

    //create the itemstore
    let itemstore = crate::server::itemstore::Itemstore::new(mize_folder.clone() + "/db").await.expect("error creating itemstore");

    // A collection of all the things, that most functions need
    let mutexes: Mutexes = Mutexes {
        next_free_client_id: Arc::new(Mutex::new(0)),
        clients: Arc::new(Mutex::new(Vec::new())),
        subs: Arc::new(Mutex::new(HashMap::new())),
        modules: Arc::new(Mutex::new(Vec::new())),
        itemstore: Arc::new(Mutex::new(itemstore)),
    };

    //listen on the local unix socket
    //local_socket_server(mize_folder);

    //run the webserver
    warp_server(mize_folder, mutexes).await;
}


async fn warp_server(mize_folder: String, mutexes: Mutexes) {
    /*
     *
     *
     */

    //### Main Endpoints:
    //## any number (with "-" in them) is interpreted as an item id
    //## $api/socket/id
    //## $api/rest/
 
    // get itemstore

    let mutexes_clone = Mutexes::clone(&mutexes);

    let socket_route = warp::path!("$api" / "socket")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let mutexes_clone = Mutexes::clone(&mutexes_clone);
            ws.on_upgrade(move |socket| handle_websocket_connection(socket, mutexes_clone))
    });

    let render_route = warp::path!("$api" / "render").map(move || "render route");
    let file_route = warp::path!("$api" / "file").map(move || "file route");

    //temporary
    let render_route = warp::path!("$api" / "render" / "first").map(move || {
        let answer = fs::read_to_string("/home/sebastian/work/mize/first-render/src/main.js").unwrap();
        warp::http::Response::builder().header("content-type", "text/javascript").body(answer)
    });

    let main_route = warp::path!("$api" / "client" / "main.js").map(move || {
        let answer = fs::read_to_string("/home/sebastian/work/mize/js-client/src/main.js").unwrap();
        warp::http::Response::builder().header("content-type", "application/javascript").body(answer)
    });

    let defuatl_route = warp::path::full().map(|path|{
        //normally should be included in the binary, but for developing just gonna read the file
        //so I don't have to recompile all the time
        //let answer = include_str!("../js-client/src/main.html");
        let answer = fs::read_to_string("/home/sebastian/work/mize/js-client/src/main.html").unwrap();
        warp::http::Response::builder().body(answer)
    });
    
    //let routes = warp::get().and(socket_route).or(render_route).or(file_route).map(move |hi| "default");
    let routes = warp::get().and(
        
        render_route
            .or(file_route)
            .or(socket_route)
            .or(main_route)
            .or(defuatl_route)
            //.or(sum)
            //.or(times),
    );
       
    warp::serve(routes).run(([127, 0, 0, 1], 3000)).await;
}

async fn handle_websocket_connection(
    socket: WebSocket,
    mutexes: Mutexes,
){
    let (mut socket_tx, mut socket_rx) = socket.split();
    let (msg_tx, msg_rx): (Sender<proto::Message>, Receiver<proto::Message>) = mpsc::channel(SOCKET_CHANNEL_SIZE);

    let mut msg_rx = ReceiverStream::new(msg_rx);

//    tokio::spawn(msg_rx.forward(socket_tx));

    //my own forward
    tokio::spawn(async move {
        while let Some(msg) = msg_rx.next().await {
            socket_tx.send(warp::ws::Message::binary(msg.raw)).await;
        }
    });

    let mut cli = mutexes.clients.lock().await;
    let mut client_id = mutexes.next_free_client_id.lock().await;
    let client = Client{tx: msg_tx.clone(), id: *client_id};
    cli.push(client.clone());
    *client_id += 1;
    drop(cli);
    drop(client_id);

    // Reading and broadcasting messages
    while let Some(result) = socket_rx.next().await {
        let msg = result.expect("Error when getting message from WebSocket");
        println!("got message: {:?}", msg);

        if let Err(err) = handle_mize_message(
            proto::Message::from_bytes(msg.clone().into_bytes(),proto::Origin::Client(client.clone())), mutexes.clone(),
        ).await {
            msg_tx.send(err.to_message(proto::Origin::Client(client.clone()))).await;
        };
    };
}

//pub async fn handle_unix_socket_connection(){
//}

//async fn handle_connection(con: Connection){
//}

//pub async fn send_to_all_clients(message: ws::Message, mutexes_clone: Mutexes){
//    let clients = mutexes_clone.clients.lock().await;
//    for client in &clients.clients[..] {
//        client.tx.send(Ok(ws::Message::binary(message.clone())));
//    }
//    drop(clients);

//    let modules = mutexes_clone.modules.lock().await;
//    for module in &modules.modules[..] {
//        module.tx.send(Ok(ws::Message::binary(message.clone())));
//    }
//}

//pub async fn send_to_all_subbed_clients(id: String, message: ws::Message, mutexes_clone: Mutexes){
//    let clients = mutexes_clone.clients.lock().await;
//    for client in &clients.clients[..] {
//        if client.sub.contains(&id) {
//            client.tx.send(Ok(ws::Message::binary(message.clone())));
//        }
//    }

//    let modules = mutexes_clone.modules.lock().await;
//    for module in &modules.modules[..] {
//        if module.sub.contains(&id) {
//            module.tx.send(Ok(ws::Message::binary(message.clone())));
//        }
//    }
//}

//that does not work like this

//impl From<proto::Message> for Result<warp::ws::Message, warp::Error> {
//    fn from(msg: proto::Message) -> Result<warp::ws::Message, warp::Error> { 
//        let msg = warp::ws::Message::binary(msg.raw);
//    }
//}

//impl From<Result<warp::ws::Message, warp::Error>> for proto::Message {
//    fn from(ws_msg: Result<warp::ws::Message, warp::Error>) -> proto::Message {
//        let msg = ws_msg.expect("Error in From trait implementation: warp::ws::Messate to proto::Message");
//    }
//}
//191 |     cli.push(Client{tx: client_tx, id: client_id});
//    = note: expected struct `tokio::sync::mpsc::Sender<proto::Message>`
//               found struct `tokio::sync::mpsc::Sender<`



