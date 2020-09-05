use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;

pub struct VoteObj {
    pub has_started: bool,
    pub time_left: i64,
    pub times: HashMap<String, i64>,
    pub has_local_file: bool,
    pub location: String,
}
pub struct VoteRegex {
    pub time_regex: Regex,
    pub number_regex: Regex,
    pub file_regex: Regex,
    pub vote_regex: Regex,
    pub help_regex: Regex,
}
pub struct ChatMessage {
    pub user: String,
    pub roles: Elevation,
    pub text: String,
    pub channel: String,
}

pub struct Reply {
    pub user: String,
    pub text: String,
}

#[derive(Copy, Clone)]
pub enum Elevation {
    Broadcaster,
    Moderator,
    Viewer,
}

pub struct ChatCommand {
    pub command: String,
    pub elevation: Elevation,
    pub response: Box<dyn FnMut(String, String) -> String>,
    pub help: String,
    pub repeat_interval: i64,
}

unsafe impl Send for ChatCommand {}
unsafe impl Sync for ChatCommand {}

impl ChatCommand {
    pub fn new(
        c: String,
        e: Elevation,
        r: Box<dyn FnMut(String, String) -> String>,
        h: String,
        ri: i64,
    ) -> ChatCommand {
        ChatCommand {
            command: c,
            elevation: e,
            response: r,
            help: h,
            repeat_interval: ri,
        }
    }
}

impl VoteObj {
    pub fn new(duration: i64, location: String) -> VoteObj {
        let mut has_local_file = false;
        if location.as_str() != "" {
            has_local_file = true;
        }
        VoteObj {
            has_started: false,
            time_left: duration,
            times: HashMap::new(),
            has_local_file: has_local_file,
            location: location,
        }
    }
    pub fn add_vote(&mut self, username: String, time: i64) {
        //covers both adding new times and updating existing ones
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
    pub fn update_sheet(&self) {}
    pub fn rewite_sheet(&self) {}
    pub fn read_sheet(&self) {}
}
impl VoteRegex {
    pub fn new() -> VoteRegex {
        let time_regex = regex::Regex::new(r"((\d+:)?[0-5]\d:)[0-5]\d").unwrap();
        let number_regex = regex::Regex::new(r":{0}\d+").unwrap();
        let file_regex = regex::Regex::new(r"(?:\d )\w*").unwrap();
        let vote_regex = regex::Regex::new(r"(?:vote )\w*(?: \d)").unwrap();
        let help_regex = regex::Regex::new(r"(?:help )(\w+\b)").unwrap();

        VoteRegex {
            time_regex: time_regex,
            number_regex: number_regex,
            file_regex: file_regex,
            vote_regex: vote_regex,
            help_regex: help_regex,
        }
    }
}
