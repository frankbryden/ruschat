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

    fn start_typing(&mut self) {
        self.is_typing = true;
    }
    
    fn stop_typing(&mut self) {
        self.is_typing = false;
    }

    pub fn get_addr(&self) -> &SocketAddr {
        &self.addr
    }

    pub fn get_tx(&self) -> &Sender<Event> {
        &self.tx
    }
}