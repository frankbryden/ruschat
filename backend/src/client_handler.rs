use futures::{self};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::thread::spawn;
use std::time::{Duration, SystemTime};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

use crate::emojis::EmojisHolder;
use crate::event::{Event, EventKind};
use crate::user::build_user;
use crate::{PeerMap, ThreadSafeDbHandler};

fn handle_other_client_messages(
    mut tx: SplitSink<WebSocketStream<TcpStream>, Message>,
    other_client_messages_rx: Receiver<Event>,
    _db_handler: ThreadSafeDbHandler,
) {
    for event in other_client_messages_rx {
        futures::executor::block_on(tx.send(Message::text(event.to_client_message()))).unwrap();
    }
}

fn handle_incoming_messages(
    rx: SplitStream<WebSocketStream<TcpStream>>,
    state: PeerMap,
    my_addr: SocketAddr,
    db_handler: ThreadSafeDbHandler,
    emojis_holder: Arc<EmojisHolder>,
) {
    let mut my_name = String::new();
    let future = rx.for_each(|message| {
        let message = match message {
            Ok(m) => m,
            Err(e) => {
                panic!("Failed to read message: {e}");
            }
        };
        if message.is_text() {
            //Convert the message from a Message to a str ref
            let text = message.to_text().unwrap();
            let event: Option<Event>;

            if text.starts_with("user:") {
                println!("ClientLogin branch");

                //Lock the current state
                let mut s = state.lock().unwrap();

                //Get the username from the message
                let user_info = text.split_once(":").unwrap().1;
                // TODO this will panic when the login message is invalid
                let (username, profile_pic) = user_info.split_once("#").unwrap();
                println!("User login: {username}");

                //Get ref to my user
                let me = s.get_mut(&my_addr).unwrap();

                //Create username string and set for current user
                my_name = String::from(username);
                me.set_name(my_name.clone());
                me.set_profile_pic(String::from(profile_pic));
                me.login();

                let sender = me.get_tx().clone();
                sender
                    .send(Event::new(
                        EventKind::LobbyState(
                            s.values()
                                .cloned()
                                .filter(|user| user.is_logged_in())
                                .collect::<Vec<_>>(),
                        ),
                        None,
                    ))
                    .unwrap();

                event = Some(Event::new(
                    EventKind::ClientLogin((
                        my_name.clone(),
                        s.values()
                            .cloned()
                            .filter(|user| user.is_logged_in())
                            .collect::<Vec<_>>(),
                    )),
                    None,
                ));
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

                event = Some(Event::new(
                    EventKind::Typing(
                        s.values()
                            .cloned()
                            .filter(|u| u.is_typing())
                            .collect::<Vec<_>>(),
                    ),
                    None,
                ));
            } else if text.starts_with("history:") {
                let timestamp: u64 = text.split_once(":").unwrap().1.parse().unwrap();
                let db = db_handler.lock().unwrap();
                let events = db.read_events(
                    SystemTime::UNIX_EPOCH + Duration::from_millis(timestamp),
                    10,
                );

                let s = state.lock().unwrap();
                let sender = s.get(&my_addr).unwrap().get_tx();
                for event in events {
                    sender.send(event).unwrap();
                }
                event = None;
            } else if text.starts_with("emojis:") {
                let query = text.split_once(":").unwrap().1;
                let emojis = emojis_holder.query(query);
                let s = state.lock().unwrap();
                let sender = s.get(&my_addr).unwrap().get_tx();
                let response = emojis
                    .iter()
                    .map(|(name, url)| format!("{};{}", name, url))
                    .collect::<Vec<String>>()
                    .join(":");
                sender
                    .send(Event::new(EventKind::EmojiQuery(response), None))
                    .unwrap();
                event = None;
            } else {
                event = Some(Event::new(
                    EventKind::ClientMessage((my_name.clone(), message.clone())),
                    None,
                ));
            }

            let mut handle = db_handler.lock().unwrap();
            if event.is_some() {
                let event = event.unwrap();
                handle.record_event(event.clone());
                //Perform the sending
                let s = state.lock().unwrap();
                //Send to everyone but my user
                let keys_to_send: Vec<&SocketAddr> =
                    s.keys().filter(|&&addr| addr != my_addr).collect();
                for key in keys_to_send {
                    match s.get(key).unwrap().get_tx().send(event.clone()) {
                        Ok(_) => {}
                        Err(e) => println!("Failed to send message to {key}, got {e}"),
                    }
                }
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
    if username.len() > 0 {
        for value in s.values() {
            value
                .get_tx()
                .send(Event::new(
                    EventKind::ClientLogout((
                        username.clone(),
                        s.values()
                            .cloned()
                            .filter(|user| user.is_logged_in())
                            .collect::<Vec<_>>(),
                    )),
                    None,
                ))
                .unwrap();
        }
    }

    println!("Done reading messages, dropped {username}");
}

pub async fn handle_client(
    state: PeerMap,
    stream: TcpStream,
    addr: SocketAddr,
    db_handler: ThreadSafeDbHandler,
    emojis_holder: Arc<EmojisHolder>,
) {
    let addr_text = addr.to_string();
    println!("Handling new client {addr_text}");
    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    let (tx, rx) = mpsc::channel();
    state.lock().unwrap().insert(addr, build_user(addr, tx));
    let (outgoing, incoming) = ws_stream.split();
    let incoming_db_handler = db_handler.clone();
    let outgoing_db_handler = db_handler.clone();
    spawn(move || {
        // handle_incoming_messages(incoming, state.clone(), addr, db_handler.clone());
        handle_incoming_messages(
            incoming,
            state.clone(),
            addr,
            incoming_db_handler,
            emojis_holder,
        );
    });
    spawn(move || {
        // handle_other_client_messages(outgoing, rx, db_handler.clone());
        handle_other_client_messages(outgoing, rx, outgoing_db_handler);
    });
}
