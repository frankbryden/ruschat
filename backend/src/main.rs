use std::{
    collections::HashMap,
    io::Error as IoError,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::protocol::Message;
use user::User;

#[derive(Clone)]
pub enum Event {
    ClientMessage((String, Message)),
    //ClientLogin contains the name of the user that logged in, as well as the current state of the lobby
    ClientLogin((String, Vec<User>)),
}

// type PeerMap<'a> = Arc<Mutex<HashMap<SocketAddr, User<'a>>>>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, User>>>;

mod client_handler;
mod user;


#[tokio::main]
async fn main() -> Result<(), IoError> {
    let addr = "127.0.0.1:9001".to_string();

    let state = PeerMap::new(Mutex::new(HashMap::new()));

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(client_handler::handle_client(state.clone(), stream, addr));
    }

    Ok(())
}