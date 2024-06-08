use std::time::SystemTime;

use crate::user::User;
use tokio_tungstenite::tungstenite::protocol::Message;

const USER_SEPARATOR: &str = "&";

#[derive(Clone, Debug)]
pub struct Event {
    pub timestamp: u64,
    pub kind: EventKind,
    pub channel: String,
}

impl Event {
    pub fn new(kind: EventKind, channel: Option<String>) -> Event {
        Event{
            timestamp: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64,
            kind,
            channel: channel.unwrap_or(String::from("default")),
        }
    }

    pub fn to_client_message(&self) -> String {
        let message = match &self.kind {
            EventKind::ClientMessage((username, message)) => {
                format!("{}:{}", username, message.to_string())
            },
            EventKind::ClientLogin((username, users)) => {
                format!("login:{}:{}", username, get_users_str_from_vec_with_images(users))
            },
            EventKind::ClientLogout((username, users)) => {
                format!("logout:{}:{}", username, get_users_str_from_vec(users))
            },
            EventKind::LobbyState(users) => {
                println!("Sending lobby state with {} users", users.len());
                format!("lobby:{}", get_users_str_from_vec_with_images(users))
            },
            EventKind::Typing(users) => {
                println!("Currently got {} users typing", users.len());
                format!("typing:{}", get_users_str_from_vec(users))
            },
        };
        format!("{}:{}", self.timestamp, message)
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

fn get_users_str_from_vec(users: &Vec<User>) -> String {
    String::from(users.iter().map(|u| u.get_name().clone()).collect::<Vec<String>>().join(USER_SEPARATOR))
}

fn get_users_str_from_vec_with_images(users: &Vec<User>) -> String {
    String::from(users.iter().map(|u| u.get_name().clone() + "#" + &u.get_profile_pic().clone().unwrap_or_default()).collect::<Vec<String>>().join(USER_SEPARATOR))
}