use std::time::SystemTime;

use crate::user::User;
use tokio_tungstenite::tungstenite::protocol::Message;

#[derive(Clone, Debug)]
pub struct Event {
    pub timestamp: u64,
    pub kind: EventKind,
}

impl Event {
    pub fn new(kind: EventKind) -> Event {
        Event{
            timestamp: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
            kind,
        }
    }
}

#[derive(Clone, Debug)]
pub enum EventKind {
    ClientMessage((String, Message)),
    //ClientLogin contains the name of the user that logged in, as well as the current state of the lobby
    ClientLogin((String, Vec<User>)),
    ClientLogout((String, Vec<User>)),
    LobbyState(Vec<User>),
    Typing(Vec<User>),
}

pub fn event_kind_as_char(e: &EventKind) -> char {
    match *e {
        EventKind::ClientMessage(_) => 'M',
        EventKind::ClientLogin(_) => 'J', //J for join
        EventKind::ClientLogout(_) => 'L', //L for leave
        EventKind::LobbyState(_) => 'S', //S for lobby "State"
        EventKind::Typing(_) => 'T',
    }
}

pub fn get_event_username(e: &Event) -> Option<&str> {
    match &e.kind {
        EventKind::ClientMessage((user, _)) => Some(&user),
        EventKind::ClientLogin((user, _)) => Some(&user),
        EventKind::ClientLogout((user, _)) => Some(&user),
        EventKind::LobbyState(_) => None,
        EventKind::Typing(_) => None,
    }
}

pub fn get_event_text(e: &Event) -> Option<&str> {
    match &e.kind {
        EventKind::ClientMessage((_, message)) => Some(message.to_text().unwrap()),
        _ => None,
    }
}