use std::sync::{Arc};
use std::collections::HashMap;

pub struct VoteObj{
    pub has_started: bool,
    pub times: HashMap<String, i64>,
    pub has_local_file: bool,
    pub location: String
}
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


impl VoteObj{
    pub fn new(location: String) -> VoteObj{
        let mut has_local_file = false;
        if location.as_str() != ""{
            has_local_file = true;
        }
        VoteObj{has_started: false, times: HashMap::new(), has_local_file: has_local_file, location: location}
    }
    pub fn add_time(&mut self, username: String, time: i64){ //covers both adding new times and updating existing ones
        if self.times.contains_key(&username) {
            self.times.remove(&username);
            self.times.insert(username, time);
            if self.has_local_file {
                self.rewite_sheet();
            }
        } else {
            self.times.insert(username, time);
            if self.has_local_file {
                self.update_sheet();
            }
        } 
    }
    pub fn update_sheet(&self){
    }
    pub fn rewite_sheet(&self){
    }
    pub fn read_sheet(&self){
    }
}
