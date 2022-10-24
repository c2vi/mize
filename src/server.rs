

pub mod server_utils;
pub mod itemstore;
pub mod proto;

use futures_util::{FutureExt, StreamExt};
use warp::Filter;
use warp::ws::{WebSocket, self};

use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};
use tokio_stream::wrappers::UnboundedReceiverStream;
use std::alloc::handle_alloc_error;
use std::fs;
use std::sync::{Arc};
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use crate::server::proto::{Response};
use crate::server::proto::handle_mize_message;

use self::itemstore::Itemstore;

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
    tx: UnboundedSender<Result<ws::Message, warp::Error>>
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
    
    //println!("TEST");
    //testing
    //let val = "mize.works".to_string().into_bytes();
    //let mut update: Vec<u8> = Vec::new();
    //update.push(2);
    //update.extend(u32::to_be_bytes(3));
    //update.extend(u32::to_be_bytes(4));
    //update.extend("$$".to_string().into_bytes());
    
    //update.push(0);
    //update.extend(u32::to_be_bytes(5));
    //update.extend(u32::to_be_bytes(7));
    //update.extend("$$".to_string().into_bytes());

    //let new_val = proto::apply_update(&val, &update);
    //println!("TEST");
    //println!("TEST: {}", String::from_utf8(new_val).unwrap());

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
    let itemstore = crate::server::itemstore::Itemstore::new(mize_folder + "/db").await;

    let mut clients: Arc<Mutex<Vec<Client>>> = Arc::new(Mutex::new(Vec::new()));
    let mut itemstore_mutex: Arc<Mutex<Itemstore>> = Arc::new(Mutex::new(itemstore));

    let socket_route = warp::path!("$api" / "socket")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let clients_clone = Arc::clone(&clients);
            let itemstore_clone = Arc::clone(&itemstore_mutex);
            ws.on_upgrade(move |socket| handle_socket_connection(
                    socket,
                    clients_clone,
                    itemstore_clone
                )
            )
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

async fn handle_socket_connection(
    socket: WebSocket,
    clients_clone: Arc<Mutex<Vec<Client>>>,
    itemstore_clone: Arc<Mutex<Itemstore>>,
){
    let (socket_tx, mut socket_rx) = socket.split();
    let (client_tx, client_rx) = mpsc::unbounded_channel();

    let client_rx = UnboundedReceiverStream::new(client_rx);
    tokio::spawn(client_rx.forward(socket_tx));

    let mut cli = clients_clone.lock().await;
    cli.push(Client{tx: client_tx.clone()});
    drop(cli);

    // Reading and broadcasting messages
    while let Some(result) = socket_rx.next().await {
        let msg = result.expect("Error when gettin message from WebSocket");

        let itemstore = &*itemstore_clone.lock().await;
        match handle_mize_message(proto::Message::new(msg.clone().into_bytes()), itemstore).await {
            Response::One(response) => {
                client_tx.send(Ok(ws::Message::binary(response.clone()))).unwrap()
            },
            Response::All(response) => {
                let clients = clients_clone.lock().await;
                println!("Clients: {}", clients.len());
                for client in &clients[..] {
                    client.tx.send(Ok(ws::Message::binary(response.clone())));
                }
                drop(clients);
            },
            Response::None => {let t = 0;},
        };
    }
}



