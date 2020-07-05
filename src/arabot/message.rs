use std::sync::{Arc};

pub struct ChatMessage{
    pub user: String,
    pub roles: Elevation,
    pub text: String,
    pub channel: String
}

pub struct Reply{
    pub user: String,
    pub text: String
}

#[derive(Copy, Clone)]
pub enum Elevation{
    Broadcaster,
    Moderator,
    Viewer
}

pub struct ChatCommand{
    pub command: String,
    pub elevation: Elevation,
    pub response: Box<dyn FnMut(String, String) -> String>,
    pub help: String,
    pub repeat_interval: i64,
}

unsafe impl Send for ChatCommand{}
unsafe impl Sync for ChatCommand{}

impl ChatCommand{
    pub fn new(c: String, e: Elevation, r: Box<dyn FnMut(String, String) -> String>, h: String, ri: i64)->ChatCommand{
        ChatCommand{command: c, elevation: e, response: r, help: h, repeat_interval: ri}    
    }
}

