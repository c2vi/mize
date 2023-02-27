

pub mod server_utils;
pub mod itemstore;

pub mod proto;

//use futures_util::lock::Mutex;
use futures_util::{FutureExt, StreamExt, SinkExt};
use std::collections::HashMap;
use std::boxed::Box;
use std::path::Path;
use crate::error;
use crate::error::MizeError;
use crate::error::ERRORS;
use crate::server::proto::MizeMessage;

use serde_json::Value as JsonValue;
use serde::{Serialize, Deserialize};

use tokio::sync::mpsc::{Sender, channel, Receiver};
use tokio_stream::wrappers::ReceiverStream;
use std::alloc::handle_alloc_error;
use std::fs;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;

use self::itemstore::Itemstore;

use axum_extra::routing::SpaRouter;
use axum::Router;
use axum::routing::{get, get_service};
use std::net::SocketAddr;
use axum::response::{IntoResponse, Html};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{State, self};
use axum::http::{StatusCode, Uri, Response, self};
use axum::http::header::{HeaderName, HeaderValue, HeaderMap};
use tower_http::services::{ServeDir, ServeFile};



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
pub enum Peer {
    Client(Client), //the client id
    Module(Module), //Module_name
    Upstream(String), //a hostname Type, but for now just a string
}

impl Peer {
    pub async fn send<T> (&self, message: T) where T: Into<proto::MizeMessage>{
        let message: proto::MizeMessage = message.into();
        match self {
            Peer::Client(client) => {client.tx.send(message).await;},
            Peer::Module(module) => {module.tx.send(message).await;},
            Peer::Upstream(_) => {},
        }
    }
    pub fn get_id(&self) -> u64{
        match self {
            Peer::Client(client) => client.id,
            Peer::Module(module) => module.client_id,
            Peer::Upstream(_) => 0,
        }
    }
}


#[derive(Clone, Debug)]
pub struct Client {
//    rx: UnboundedReceiverStream<Result<ws::Message, warp::Error>>,
    tx: Sender<proto::MizeMessage>,
    id: u64,
}

#[derive(Clone, Debug)]
pub struct Module {
    tx: Sender<proto::MizeMessage>,
    name: String,
    client_id: u64,
    kind: ModuleKind,
    registered_types: Vec<String>,
}

#[derive(Clone, Debug)]
pub enum ModuleKind {
    Rust(),
    Python(),
    JS(),
    Lua(),

    //connects to ws server
    Extern(),
}

#[derive(Clone, Debug)]
pub struct Render {
    id: String,
    webroot: String,
    main: String,
    folder: String,
}

trait SendMize<T> where T: Into<MizeMessage> {
    fn send(&self, msg: T);
}

impl<T> SendMize<T> for Sender<MizeMessage> where MizeMessage: From<T> {
    fn send(&self, msg: T){
        self.send(msg.into());
    }
}

//pub struct Upstream maybe???? ... I'd say later....


// A collection of all the things, that most functions need
#[derive(Clone)]
pub struct Mutexes {
    next_free_client_id: Arc<Mutex<u64>>,
    clients: Arc<Mutex<Vec<Client>>>,
    modules: Arc<Mutex<Vec<Module>>>,
    subs: Arc<Mutex<HashMap<String, Vec<Peer>>>>,
    renders: Arc<Mutex<Vec<Render>>>,
    //maybe a list of upstream servers??
    itemstore: Arc<Mutex<Itemstore>>,
    mize_folder: String,
}

impl Mutexes {
    pub fn clone(mutexes: &Mutexes) -> Mutexes {
        Mutexes {
            next_free_client_id: Arc::clone(&mutexes.next_free_client_id),
            clients: Arc::clone(&mutexes.clients),
            modules: Arc::clone(&mutexes.modules),
            renders: Arc::clone(&mutexes.renders),
            subs: Arc::clone(&mutexes.subs),
            mize_folder: mutexes.mize_folder.clone(),
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

    //load modules and renders
    let ren_mods = std::fs::read_dir(mize_folder.clone() + "/mr")
        .expect("error reading the modules-renders dir in the mize-folder")
        .filter(|entry| entry.as_ref().unwrap().file_type().unwrap().is_dir());
    
    let mut renders: Vec<Render> = Vec::new();

    for ren_mod in ren_mods {
        let ren_mod = ren_mod.unwrap();
        if let Ok(toml_string) = fs::read_to_string(format!("{}/mize.toml", ren_mod.path().display())){
            let data = toml_string.parse::<toml::Value>()
                .expect(&format!("error while parsing the mize.toml file in {}", ren_mod.path().display()));

            //set renders
            let render_arr = match data.get("render").expect(&format!("something wrong in the mize.toml file in {:?}", ren_mod.file_name())) {
                toml::Value::Array(arr) => arr,
                _ => {panic!("something wrong in the mize.toml file in {:?}", ren_mod.file_name())}
            };

            for render in render_arr {
                let id = match render.get("id").expect(&format!("something wrong in the mize.toml file in {:?}", ren_mod.file_name())) {
                    toml::Value::String(val) => val,
                    _ => {panic!("something wrong in the mize.toml file in {:?}", ren_mod.file_name())}
                };

                let webroot = match render.get("webroot").expect(&format!("something wrong in the mize.toml file in {:?}", ren_mod.file_name())) {
                    toml::Value::String(val) => val,
                    _ => {panic!("something wrong in the mize.toml file in {:?}", ren_mod.file_name())}
                };

                let main = match render.get("main").expect(&format!("something wrong in the mize.toml file in {:?}", ren_mod.file_name())) {
                    toml::Value::String(val) => val,
                    _ => {panic!("something wrong in the mize.toml file in {:?}", ren_mod.file_name())}
                };

                let webroot = webroot.clone();
                let main = main.clone();
                let id = id.clone();
                let folder = format!("{}", ren_mod.file_name().into_string().expect("filename has non utf8 chars in it......"));

                renders.push(Render {id, webroot, main, folder});
            };

            //set modules
            //later...
        }
    }


    // Collection of all the things, that most functions need
    let mutexes: Mutexes = Mutexes {
        next_free_client_id: Arc::new(Mutex::new(0)),
        clients: Arc::new(Mutex::new(Vec::new())),
        subs: Arc::new(Mutex::new(HashMap::new())),
        modules: Arc::new(Mutex::new(Vec::new())),
        renders: Arc::new(Mutex::new(renders)),
        itemstore: Arc::new(Mutex::new(itemstore)),
        mize_folder: mize_folder.clone(),
    };

    //listen on the local unix socket
    //local_socket_server(mize_folder);

    //run the webserver
    axum_server(mize_folder, mutexes).await;
}


async fn axum_server(mize_folder: String, mutexes: Mutexes) {
    /*
     *
     *
     */

    //### Main Endpoints:
    //## any number (with "-" in them) is interpreted as an item id
    //## $api/socket/id
    //## $api/rest/
 


    let mutexes_clone = Mutexes::clone(&mutexes);

    let serve_client = get_service(ServeDir::new("js-client/src")).handle_error(handle_error);

    let mut app = Router::new()
        .route("/==api==/socket", get(websocket_handler))
        .route("/==api==/render/:id", get(get_render_main))
//        .route("/$api/file", get(get_file_handler))
//        .merge(SpaRouter::new("/$api/client", "js-client/src").index_file("main.html"))
        .nest_service("/==api==/client", serve_client)
        .with_state(mutexes.clone())
        .fallback(render_item);

    let renders = mutexes.renders.lock().await;

    for render in &*renders {
        let url_path = String::from("/==api==/render/") + &render.id + "/webroot";
        let file_path = mutexes.mize_folder.clone() + "/modules-renders/" + &render.folder[..] + "/" + &render.webroot.clone()[..];
        let serve_dir = get_service(ServeDir::new(&file_path)).handle_error(handle_error);

        app = app.nest_service(&url_path, serve_dir);
    }
    drop(renders);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
//    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

}

async fn handle_error(_err: std::io::Error) -> impl IntoResponse {
        (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}

async fn render_item(uri: Uri) -> impl IntoResponse {
    Html(fs::read_to_string("js-client/src/main.html").unwrap())
}

//#[axum_macros::debug_handler]
async fn get_render_main(extract::Path(id): extract::Path<String>, State(mutexes): State<Mutexes>) -> http::Response<String> {
    let renders = &*mutexes.renders.lock().await;

    let render = renders.iter().filter(|&render| render.id == id).nth(0)
        .unwrap_or(renders.iter().filter(|&render| render.id == "first").nth(0).expect("there is no first render"));

    let file_name = mutexes.mize_folder.clone() + "/mr/" + &render.folder + "/" + &render.main;
    println!("filenamd: {}", file_name);

    Response::builder()
        .header("content-type", "application/javascript")
        .status(StatusCode::OK)
        .body(fs::read_to_string(file_name).unwrap()).unwrap()
}

async fn websocket_handler(
        ws: WebSocketUpgrade,
        State(mutexes): State<Mutexes>,
        headers: HeaderMap,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_websocket_connection(socket, mutexes, headers))
}

async fn handle_websocket_connection(
    socket: WebSocket,
    mutexes: Mutexes,
    headers: HeaderMap,
){
    let (mut socket_tx, mut socket_rx) = socket.split();
    let (msg_tx, msg_rx): (Sender<proto::MizeMessage>, Receiver<proto::MizeMessage>) = mpsc::channel(SOCKET_CHANNEL_SIZE);

    let mut msg_rx = ReceiverStream::new(msg_rx);

    //my own forward
    tokio::spawn(async move {
        while let Some(msg) = msg_rx.next().await {
            match msg {
                proto::MizeMessage::Json(json_msg) => {
                    if let Ok(msg) = serde_json::to_string(&json_msg) {
                        socket_tx.send(Message::Text(msg)).await;
                    } else {
                        let err: MizeError = MizeError::new(11).extra_msg("error while serializing a json message").handle();
                        if let Ok(text) = serde_json::to_string(&err.to_json()){
                            socket_tx.send(Message::Text(text));
                        } else {
                            MizeError::new(11).extra_msg("not even being able to serialize an error, while failing to serializing a message").handle();
                        }
                    }
                }
                proto::MizeMessage::Bin(bin_msg) => {
                    socket_tx.send(Message::Binary(bin_msg.raw)).await;
                }
            }
        }
    });

    //check if mize-module header exists
    let origin = if let Some(mod_header_val) = headers.get("mize-module"){

        let mut mods = mutexes.modules.lock().await;
        let mut client_id = mutexes.next_free_client_id.lock().await;
        let mut module = Module{
            tx: msg_tx.clone(),
            client_id: *client_id,
            name: String::from("tmp"),
            kind: ModuleKind::Extern(),
            registered_types: Vec::new(),
        };

        *client_id += 1;

        if let Ok(name) = mod_header_val.to_str() {
            module.name = name.to_string();
        } else {
            let err = MizeError::new(113)
                .extra_msg("The mize-module Header could not be decoded into a String.");

            msg_tx.send(proto::MizeMessage::Json(err.to_json_message())).await;
            return;
        };
        println!("module {} connected", module.name);
        mods.push(module.clone());
        drop(mods);
        Peer::Module(module)

    } else {
        let mut cli = mutexes.clients.lock().await;
        let mut client_id = mutexes.next_free_client_id.lock().await;
        let client = Client{tx: msg_tx.clone(), id: *client_id};
        cli.push(client.clone());
        *client_id += 1;
        drop(cli);
        Peer::Client(client)
    };

    // Reading messages
    while let Some(result) = socket_rx.next().await {
        let msg = if let Ok(msg) = result {msg} else {
            let err = MizeError::new(115).handle();
            msg_tx.send(proto::MizeMessage::Json(err.to_json_message())).await;
            continue;
        };

        match msg {
            Message::Binary(b) => {
                println!("Recieved a Binary Message. Those are not implemented yet.")
            },


            Message::Text(text) => {
                let json_msg: proto::JsonMessage = serde_json::from_str(&text).expect("error parsing json");
                match json_msg {
                    proto::JsonMessage::ForItem(item_msg) => {
                        if let Err(err) = proto::handle_item_msg(item_msg, origin.clone(), mutexes.clone()).await {
                            let err_msg: MizeMessage = err.handle().into();
                            origin.send(err_msg).await;
                        };
                    },
                    _ => {println!("Got a non ItemMessage")},
                }

            }


            Message::Close(_) => {
                match origin {
                    Peer::Client(ref client) => {
                        //remove client from client list
                        let mut clients = mutexes.clients.lock().await;
                        let index = match clients.iter()
                            .position(|client_iter| client_iter.id == client.id)
                            .ok_or(MizeError::new(11).extra_msg("A Client disconected, that had never connected......")){
                                Ok(index) => index,
                                Err(err) => {
                                    let msg: MizeMessage = err.into();
                                    origin.send(msg);
                                    return;
                                },
                            };
                        clients.remove(index);
                        println!("Close from Client");
                        return;
                    },
                    Peer::Module(module) => {
                        //remove module from module list
                        let mut modules = mutexes.modules.lock().await;
                        let index = modules.iter()
                            .position(|module_iter| module_iter.client_id == module.client_id)
                            .expect("A Module disconected, that had never connected.....");
                        modules.remove(index);
                        println!("Close from Module");
                        return;
                    }
                    Peer::Upstream(_) => {
                        return;
                    }
                }
            }


            _ => {
                let err = MizeError::new(11)
                    .extra_msg("unhandeld WebSocket-Message type")
                    .handle();

                msg_tx.send(proto::MizeMessage::Json(err.to_json_message())).await;
            },
        };
    };
}



