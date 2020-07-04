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

pub struct ChatCommand<F: FnMut(String, String) -> String + 'static>{
    pub command: String,
    pub elevation: Elevation,
    pub response: Arc<F>,
    pub help: String,
    pub repeat_interval: i64,
}

unsafe impl<F: FnMut(String, String) -> String> Send for ChatCommand<F> {}
unsafe impl<F: FnMut(String, String) -> String> Sync for ChatCommand<F> {}

impl<F: FnMut(String, String) -> String + 'static> ChatCommand<F>{
    pub fn new(c: String, e: Elevation, r: Arc<F>, h: String, ri: i64)->ChatCommand<F>{
        ChatCommand{command: c, elevation: e, response: r, help: h, repeat_interval: ri}    
    }
}

