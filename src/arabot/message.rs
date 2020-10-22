use regex::Regex;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::{thread, time, fs::OpenOptions, io::Write};
use std::{path::Path, fs};
use fs::File;

pub struct VoteObj {
    pub has_started: Mutex<bool>,
    pub time_left: Mutex<Vec<u64>>,
    pub times: Mutex<HashMap<String, String>>,
    //pub times_str: Mutex<HashMap<String, String>>,
    pub has_local_file: bool,
    pub active_thread: Mutex<thread::JoinHandle<()>>,
    pub location: String,
}
pub struct VoteRegex {
    pub time_regex: Regex,
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
    pub fn new(duration: u64, location: String) -> Arc<VoteObj> {
        let mut has_local_file = false;
        let mut time_left: Vec<u64> = Vec::new();
        let mut file_name = String::from("");
        let local_thread = thread::spawn(move || {});
        if location.as_str() != "" {
            has_local_file = true;
            file_name = location + ".csv";
            fs::write(String::from(&file_name),"Name;Result\n");
        }
        time_left.push(duration);
        Arc::new(VoteObj {
            has_started: Mutex::new(false),
            time_left: Mutex::new(time_left),
            times: Mutex::new(HashMap::new()),
            has_local_file: has_local_file,
            active_thread: Mutex::new(local_thread),
            location: file_name,
        })
    }
    pub fn start_vote(current_session: Arc<VoteObj>) {
        *current_session.has_started.lock().unwrap() = true;
        while current_session.time_left.lock().unwrap().len() != 0 {
            let waiting_time = current_session.time_left.lock().unwrap().pop().unwrap();
            thread::park_timeout(time::Duration::from_secs(
                waiting_time,
            ));
        }
        *current_session.has_started.lock().unwrap() = false;
    }
    pub fn add_vote(&self, username: String, time: String) {
        //covers both adding new times and updating existing ones
        if self.times.lock().unwrap().contains_key(&username) {
            self.times.lock().unwrap().remove(&username);
            self.times.lock().unwrap().insert(username, time);
            if self.has_local_file {
                self.rewrite_sheet();
            }
        } else {
            self.times.lock().unwrap().insert(String::from(&username), String::from(&time));
            if self.has_local_file {
                self.update_sheet(username, time);
            }
        }
    }
    pub fn update_sheet(&self, username: String, time: String) {
        let path = Path::new(&self.location);

        let result_file = OpenOptions::new().read(true).append(true).open(&path);

        if let Err(e) = writeln!(result_file.unwrap(), "{};{}", username, time) {
            eprintln!("Couldn't write to file: {}", e);
        }
    }
    pub fn rewrite_sheet(&self) {}
    pub fn read_sheet(&self) {}
}
impl VoteRegex {
    pub fn new() -> VoteRegex {
        let time_regex = regex::Regex::new(r"(((\d+:)?[0-5]?\d:)[0-5]\d( |$))| \d+( |$)").unwrap();
        let file_regex = regex::Regex::new(r"(?:\d )(\w*)").unwrap();
        let vote_regex = regex::Regex::new(r"(?:vote )(\w*)(?: \d)").unwrap();
        let help_regex = regex::Regex::new(r"(?:help )(\w+\b)").unwrap();

        VoteRegex {
            time_regex: time_regex,
            file_regex: file_regex,
            vote_regex: vote_regex,
            help_regex: help_regex,
        }
    }
}
