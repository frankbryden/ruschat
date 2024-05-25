use std::net::SocketAddr;
use std::sync::mpsc::Sender;

use crate::Event;

#[derive(Clone, Debug)]
pub struct User {
    addr: SocketAddr,
    name: String,
    is_typing: bool,
    tx: Sender<Event>,
}

pub fn build_user(addr: SocketAddr, tx: Sender<Event>) -> User {
    User {
        addr,
        tx,
        name: String::new(),
        is_typing: false,
    }
}

impl User {
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn start_typing(&mut self) {
        self.is_typing = true;
    }
    
    pub fn stop_typing(&mut self) {
        self.is_typing = false;
    }

    pub fn is_typing(&self) -> bool {
        self.is_typing
    }

    pub fn get_addr(&self) -> &SocketAddr {
        &self.addr
    }

    pub fn get_tx(&self) -> &Sender<Event> {
        &self.tx
    }
}