

pub mod server_utils;
pub mod itemstore;

use futures_util::{FutureExt, StreamExt};
use warp::Filter;
use warp::ws::{WebSocket, Message};

use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};
use tokio_stream::wrappers::UnboundedReceiverStream;
use std::alloc::handle_alloc_error;
use std::sync::{Arc};
use tokio::sync::Mutex;
use tokio::sync::mpsc;

//static API_PATH_STRING: &str = "$api";

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
//

struct Client {
    tx: UnboundedSender<Result<Message, warp::Error>>
}

pub fn run_server(args: Vec<String>) {
    /*
     * WHAT IT DOES
     *
     */

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

    //### prep the mize_folder
    //if let Err(err) = crate::server::server_utils::init_mize_folder(mize_folder) {
        //println!("{}", err.message);
    //}
    

    //run the webserver







    warp_server(mize_folder);
}


#[tokio::main]
async fn warp_server(mize_folder: String) {
    /*
     *
     *
     */

    //### Main Endpoints:
    //## any number (with "-" in them) is interpreted as an item id
    //## $api/socket/id
    //## $api/rest/
 
    // get itemstore
    let itemstore = crate::server::itemstore::itemstore::new(mize_folder + "/db").await;

    let mut clients: Arc<Mutex<Vec<Client>>> = Arc::new(Mutex::new(Vec::new()));

    let socket_route = warp::path!("$api" / "socket")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let clients_clone = Arc::clone(&clients);
            ws.on_upgrade(move |socket| handle_socket_connection(socket, clients_clone) )
    });

    //let temp = warp::path("temp")
        //.and(warp::ws())
        //.map(move |ws: warp::ws::Ws| {
            //ws.hi();
    //});
    
    //let routes = warp::any().map(|| "Hello, World!");
    let echo = warp::path("echo").map(|| "Hello World");
    let test = warp::path("test").map(|| {

        let int = 4+4;
        format!("{}", int)
    });

    let routes = warp::get().and(socket_route).or(echo).or(test);
       
    warp::serve(routes).run(([127, 0, 0, 1], 3000)).await;
}

async fn handle_socket_connection(
    socket: WebSocket,
    clients_clone: Arc<Mutex<Vec<Client>>>
    //test: String
){
    let (socket_tx, mut socket_rx) = socket.split();
    let (client_tx, client_rx) = mpsc::unbounded_channel();

    let client_rx = UnboundedReceiverStream::new(client_rx);
    tokio::spawn(client_rx.forward(socket_tx));

    let mut cli = clients_clone.lock().await;
    cli.push(Client{tx: client_tx});
    drop(cli);

    // Reading and broadcasting messages
    while let Some(result) = socket_rx.next().await {
        let msg = result.expect("Error when gettin message from WebSocket");
        let clients = clients_clone.lock().await;
        for client in &clients[..] {
            client.tx.send(Ok(msg.clone()));
        }
        drop(clients);
    }
}



