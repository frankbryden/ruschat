use std::borrow::Borrow;
use std::sync::mpsc::Receiver;
use std::sync::mpsc;
use std::net::SocketAddr;
use futures_util::stream::{SplitSink, SplitStream};
use futures::{self, future};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::WebSocketStream;
use std::thread::spawn;

use crate::{PeerMap, Event};
use crate::user::{User, build_user};

fn get_logged_in_users_str(users: Vec<User>) -> String {
    String::from(users.iter().map(|u| u.get_name().clone()).collect::<Vec<String>>().join(","))
}

fn handle_other_client_messages(mut tx: SplitSink<WebSocketStream<TcpStream>, Message>, other_client_messages_rx: Receiver<Event>) {
    for event in other_client_messages_rx {
        let future;
        match event {
            Event::ClientMessage((username, message)) => {
                future = tx.send(Message::text(format!("{}: {}", username, message.to_string())));
                println!("Sending {message}");
            },
            Event::ClientLogin((username, users)) => {
                future = tx.send(Message::text(format!("login:{}:{}", username, get_logged_in_users_str(users))));
                println!("Sending login by {username}");
            },
            Event::ClientLogout((username, users)) => {
                future = tx.send(Message::text(format!("logout:{}:{}", username, get_logged_in_users_str(users))));
                println!("Sending logout by {username}");
            },
            Event::LobbyState(users) => {
                println!("Sending lobby state with {} users", users.len());
                future = tx.send(Message::text(format!("lobby:{}", get_logged_in_users_str(users))));
            },
            Event::Typing(users) => {
                println!("Currently got {} users typing", users.len());
                future = tx.send(Message::text(format!("typing:{}", get_logged_in_users_str(users))));
            },
        }
        futures::executor::block_on(future).unwrap();
    }
}

fn handle_incoming_messages(rx: SplitStream<WebSocketStream<TcpStream>>, state: PeerMap, my_addr: SocketAddr) {
    let mut my_name = String::new();
    let future = rx.for_each(|message| {
        let message = match message {
            Ok(m) => m,
            Err(e) => {
                panic!("Failed to read message: {e}");
            }
        };
        println!("Message: {message:?}");
        if message.is_text() {

            //Convert the message from a Message to a str ref
            let text = message.to_text().unwrap();
            let event: Event;

            if text.starts_with("user:") {
                println!("ClientLogin branch");

                //Lock the current state
                let mut s = state.lock().unwrap();
                        
                //Get the username from the message
                let username = text.split_once(":").unwrap().1;
                println!("User login: {username}"); 

                //Get ref to my user
                let me = s.get_mut(&my_addr).unwrap();

                //Create username string and set for current user
                my_name = String::from(username);
                me.set_name(my_name.clone());

                let sender = me.get_tx().clone();
                sender.send(Event::LobbyState(s.values().cloned().collect::<Vec<_>>())).unwrap();

                event = Event::ClientLogin((my_name.clone(), s.values().cloned().collect::<Vec<_>>()));
            } else if text.starts_with("typing:") {
                //Lock the current state
                let mut s = state.lock().unwrap();

                //Get our user to mutate it
                let me = s.get_mut(&my_addr).unwrap();

                let typing_state_change = text.split_once(":").unwrap().1;
                match typing_state_change {
                    "start" => me.start_typing(),
                    "stop" => me.stop_typing(),
                    _ => panic!("Invalid typing state change: {typing_state_change}"),
                };

                event = Event::Typing(s.values().cloned().filter(|u| u.is_typing()).collect::<Vec<_>>());
            } else {
                println!("ClientMessage branch");
                event = Event::ClientMessage((my_name.clone(), message.clone()));
            }

            //Perform the sending
            let s = state.lock().unwrap();
            //Send to everyone but my user
            let keys_to_send: Vec<&SocketAddr> = s.keys().filter(|&&addr| addr != my_addr).collect();
            println!("Sending {event:?} to {} people", keys_to_send.len());
            for key in keys_to_send {
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

    for value in s.values() {
        value.get_tx().send(Event::ClientLogout((username.clone(), s.values().cloned().collect::<Vec<_>>()))).unwrap();
    }

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
