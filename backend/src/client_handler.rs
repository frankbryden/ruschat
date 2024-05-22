use std::sync::mpsc::Receiver;
use std::sync::mpsc;
use std::net::SocketAddr;
use futures_util::stream::{SplitSink, SplitStream};
use futures;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::WebSocketStream;
use std::thread::spawn;

use crate::{PeerMap, Event};
use crate::user::{User, build_user};

fn get_logged_in_users_str(users: Vec<User>) -> String {
    let mut output = String::new();
    for user in users {
        output += user.get_name();
    }
    output
}


fn handle_other_client_messages(mut tx: SplitSink<WebSocketStream<TcpStream>, Message>, other_client_messages_rx: Receiver<Event>) {
    for event in other_client_messages_rx {
        match event {
            Event::ClientMessage((username, message)) => {
                let future = tx.send(Message::text(format!("{}: {}", username, message.to_string())));
                println!("Sending {message}");
                futures::executor::block_on(future).unwrap();
            },
            Event::ClientLogin((username, users)) => {
                let future = tx.send(Message::text(format!("login:{}:{}", username, get_logged_in_users_str(users))));
                println!("Sending login by {username}");
                futures::executor::block_on(future).unwrap();
            }
        }
    }
}

fn handle_incoming_messages(rx: SplitStream<WebSocketStream<TcpStream>>, state: PeerMap, my_addr: SocketAddr) {
    
    let future = rx.for_each(|message| {
        let message = message.unwrap();
        if message.is_text() {
            println!("Message: {message:?}");

            //Convert the message from a Message to a str ref
            let text = message.to_text().unwrap();
            let event: Event;
            
            //Lock the current state
            let s = state.lock().unwrap();

            //Get ref to my user
            let me = s.get(&my_addr).unwrap();

            //Send to everyone but my user
            let keys_to_send: Vec<&SocketAddr> = s.keys().filter(|&&addr| addr != *me.get_addr()).collect();
            if text.starts_with("user:") {
                //Lock the current state
                let mut s = state.lock().unwrap();

                //Get the username from the message
                let username = text.split_once(":").unwrap().1;

                //Get ref to my user
                let me = s.get_mut(&my_addr).unwrap();

                //Create username string and set for current user
                let username = String::from(username);
                me.set_name(username.clone());

                event = Event::ClientLogin((username, s.values().cloned().collect::<Vec<_>>()));
            } else {
                event = Event::ClientMessage((me.get_name().to_string(), message.clone()));
            }
            //Perform the sending
            for key in keys_to_send {
                let key_text = key.to_string();
                println!("Sending to {key_text}");
                s.get(key).unwrap().get_tx().send(event.clone()).unwrap();
            }
        }
        futures::future::ready(())
    });
    futures::executor::block_on(future);
    let mut s = state.lock().unwrap();
    let username;
    {
        let user;
        user = s.get(&my_addr).unwrap();
        username = user.get_name().clone();
    }
    s.remove(&my_addr);

    println!("Done reading messages, dropped {username}");
}

pub async fn handle_client(state: PeerMap, stream: TcpStream, addr: SocketAddr) {
    let addr_text = addr.to_string();
    println!("Handling new client {addr_text}");
    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    let (tx , rx) = mpsc::channel();
    state.lock().unwrap().insert(addr, build_user(addr, tx));
    let (outgoing, incoming) = ws_stream.split();
    spawn(move || {
        handle_incoming_messages(incoming, state.clone(), addr);
    });
    spawn(move || {
        handle_other_client_messages(outgoing, rx);
    });
}
