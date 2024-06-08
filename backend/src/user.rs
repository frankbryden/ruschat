use std::net::SocketAddr;
use std::sync::mpsc::Sender;

use crate::event::Event;

#[derive(Clone, Debug)]
pub struct User {
    addr: SocketAddr,
    name: String,
    is_typing: bool,
    tx: Sender<Event>,
    //profile_pic is an optional utf-8 encoded image
    profile_pic: Option<String>,
    logged_in: bool,
}

pub fn build_user(addr: SocketAddr, tx: Sender<Event>) -> User {
    User {
        addr,
        tx,
        name: String::new(),
        is_typing: false,
        profile_pic: None,
        logged_in: false,
    }
}

impl User {
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn login(&mut self) {
        self.logged_in = true;
    }

    pub fn is_logged_in(&self) -> bool {
        self.logged_in
    }

    pub fn set_profile_pic(&mut self, profile_pic: String) {
        self.profile_pic = Some(profile_pic);
    }

    pub fn get_profile_pic(&self) -> &Option<String> {
        &self.profile_pic
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