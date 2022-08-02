
use ws::listen;

pub fn run_server(args: Vec<String>){
    println!("running server!");
}


fn ws() {
    //A WebSocket echo server
    listen("127.0.0.1:3003", |out| {
        move |msg| {
            println!("recieved: {}", msg);
            out.send(format!("You sent me: {}", msg))
        }
    }).unwrap();
}
        
