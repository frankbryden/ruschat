mod client_handler;
mod user;
mod db;
mod event;

use std::{
    collections::HashMap,
    io::Error as IoError,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::net::TcpListener;
use user::User;
use db::DbHandler;


type PeerMap = Arc<Mutex<HashMap<SocketAddr, User>>>;
type ThreadSafeDbHandler = Arc<Mutex<DbHandler>>;

#[tokio::main]
async fn main() -> Result<(), IoError> {
    let addr = "0.0.0.0:9001".to_string();

    let state = PeerMap::new(Mutex::new(HashMap::new()));
    let db_handler = ThreadSafeDbHandler::new(Mutex::new(DbHandler::new()));
    {
        let mut handle = db_handler.lock().unwrap();
        handle.initialize();
    }

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(client_handler::handle_client(state.clone(), stream, addr, db_handler.clone()));
        println!("Added connection, state size: {}", state.lock().unwrap().len());
    }

    Ok(())
}