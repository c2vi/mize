
static SERVER_PORT: u16 = 3000;
static API_ENDPOINT: &str = "@mize";


pub fn run() -> Result<Web, MizeError> {
    /*
     *  - Run a Webserver
     *
     *
     *
     */

 
    // I will deal with renders later. cli only for now
    // let renders = 

//    for render in &*renders {
//        match render.render_type {
//            RenderType::JsonUi => {},
//            RenderType::WebComponent => {
//                let url_path = String::from("/api/render/") + &render.name + "/webroot";
//                let file_path = mutexes.mize_folder.clone() + "/mr/" + &render.folder[..] + "/" + &render.webroot.clone()[..];
//                let serve_dir = get_service(ServeDir::new(&file_path)).handle_error(handle_error);

//                app = app.nest_service(&url_path, serve_dir);
//            },
//        }
//    }


    // let serve_client = get_service(ServeDir::new("js-client/src")).handle_error(handle_error);

    let mut app = Router::new()
        .route(format!("/{}/socket", API_ENDPOINT), get(websocket_handler))
//        .route(format!("/{}/render/:id", API_ENDPOINT), get(get_render_main))
//        .route(format!("/{}/file", API_ENDPOINT), get(get_file_handler))
//        .merge(SpaRouter::new(format!("/{}/client", API_ENDPOINT), "js-client/src").index_file("main.html"))
//        .nest_service(format!("/{}/client", API_ENDPOINT), serve_client)
        .with_state(main)
        .fallback(render_item);

    let addr = SocketAddr::from(([0, 0, 0, 0], SERVER_PORT));
    // tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect(format!("Unable to bind axum::Server to {}", addr));
}

pub struct Web {
}

impl Web {

}


async fn handle_error(_err: std::io::Error) -> impl IntoResponse {
        MizeError::new(0).msg("internal server error").handle()
        (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}

async fn render_item(uri: Uri) -> impl IntoResponse {
    Html(fs::read_to_string("js-client/src/main.html").unwrap())
}

//#[axum_macros::debug_handler]
//async fn get_render_main(extract::Path(id): extract::Path<String>, State(mutexes): State<Mutexes>) -> http::Response<String> {
//    let renders = &*mutexes.renders.lock().await;

//    let render = renders.iter().filter(|&render| render.name == id).nth(0)
//        .unwrap_or(renders.iter().filter(|&render| render.name == "mize-mmejs").nth(0).expect("mize-mmejs"));

//    let file_path = mutexes.mize_folder.clone() + "/mr/" + &render.folder + "/" + &render.main;

//    match render.render_type {
//        RenderType::JsonUi => {
//            return Response::builder()
//                .header("content-type", "application/json")
//                .status(StatusCode::OK)
//                .body(fs::read_to_string(file_path).unwrap()).unwrap();
//        },
//        RenderType::WebComponent => {
//            return Response::builder()
//                .header("content-type", "application/javascript")
//                .status(StatusCode::OK)
//                .body(fs::read_to_string(file_path).unwrap()).unwrap();
//        },
//    }
//}

async fn websocket_handler(
        ws: WebSocketUpgrade,
        State(main): State<&Main>,
        headers: HeaderMap,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_websocket_connection(socket, main, headers))
}

async fn handle_websocket_connection(
    socket: WebSocket,
    main: &Main,
    headers: HeaderMap,
){
    //TODO: auth

    //socket_tx.send() ... send out the socket
    //socket_rx.read() ... read a message from the socket
    let (mut socket_tx, mut socket_rx) = socket.split();

    // msg_tx.send() ... send a MizeMessage to a Peer. should be in a Peer struct
    // msg_rx.read() ... read MizeMessages that should be sent to the Peer that this ws connection is to
    let (msg_tx, msg_rx): (Sender<proto::MizeMessage>, Receiver<proto::MizeMessage>) = mpsc::channel(SOCKET_CHANNEL_SIZE);

    // why is this actually
    let mut msg_rx = ReceiverStream::new(msg_rx);

    // check if mize-module header exists
//    let origin = if let Some(mod_header_val) = headers.get("mize-module"){

//        let mut mods = mutexes.modules.lock().await;
//        let mut client_id = mutexes.next_free_client_id.lock().await;
//        let mut module = Module{
//            tx: msg_tx.clone(),
//            client_id: *client_id,
//            name: String::from("tmp"),
//            kind: ModuleKind::Extern(),
//            registered_types: Vec::new(),
//        };

//        *client_id += 1;

//        if let Ok(name) = mod_header_val.to_str() {
//            module.name = name.to_string();
//        } else {
//            let err = MizeError::new(113)
//                .extra_msg("The mize-module Header could not be decoded into a String.");

//            msg_tx.send(proto::MizeMessage::Json(err.to_json_message())).await;
//            return;
//        };
//        println!("module {} connected", module.name);
//        mods.push(module.clone());
//        drop(mods);
//        Peer::Module(module)

//    } else {
//    };

    // Forwarding messages
    tokio::spawn(async move {
        while let Some(msg) = msg_rx.next().await {
            match msg {
                proto::MizeMessage::Json(json_msg) => {
                    if let Ok(msg) = serde_json::to_string(&json_msg) {
                        let result = socket_tx.send(Message::Text(msg.clone())).await;
                        if let Err(e) = result {
                            println!("Client no longer Connected: {:?}", e);
                            remove_peer(myorigin.clone(), mymutexes.clone()).await;
                        } else {
                            println!("SENDING: {}", msg);
                        }
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
                proto::MizeMessage::Pong() => {
                    socket_tx.send(Message::Pong(Vec::new())).await;
                }
                proto::MizeMessage::Close((num, reason)) => {
                    println!("SENDING Close message");
                    socket_tx.send(Message::Close(Some(CloseFrame{code: num, reason: Cow::Owned(reason)}))).await;
                }
            }
        }
    });

    // Reading messages
    while let Some(result) = socket_rx.next().await {
        let msg = if let Ok(msg) = result {msg} else {
            let err = MizeError::new(115).extra_msg("TCP Connection Closed");
            remove_peer(origin.clone(), mutexes.clone());
            return;
        };
        match msg {
            Message::Binary(b) => {
                println!("Recieved a Binary Message. Those are not implemented yet.")
            },


            Message::Text(text) => {
                println!("GOT: {}", text);
                let json_msg: proto::JsonMessage = serde_json::from_str(&text).expect("error parsing json");
                if let Err(err) = proto::handle_json_msg(json_msg, origin.clone(), mutexes.clone()).await {
                    let err_msg: MizeMessage = err.handle().into();
                    origin.send(err_msg).await;
                };
            },

            Message::Close(inner) => {
                match inner {
                    Some(CloseFrame{code, reason}) => {
                        origin.send(MizeMessage::Close((code, reason.into_owned()))).await;
                    },
                    None => {
                        origin.send(MizeMessage::Close((1000, "You didn't send a Close Reason".to_owned()))).await;
                    }
                };

                println!("Close from Client");
                remove_peer(origin.clone(), mutexes.clone()).await;
            }


            Message::Ping(_) => {
                msg_tx.send(MizeMessage::Pong());
            },


            Message::Pong(_) => {
                let err = MizeError::new(11)
                    .extra_msg("unhandeld WebSocket-Message type: Pong")
                    .handle();
                msg_tx.send(proto::MizeMessage::Json(err.to_json_message())).await;
            },

        };
    };
}
