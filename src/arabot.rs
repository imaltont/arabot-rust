use message::{ChatCommand, ChatMessage, Elevation, Reply, VoteObj, VoteRegex};
use std::{thread, time};
pub mod message;

use futures::prelude::*;
use irc::client;
use irc::client::prelude::*;
use irc::error::Error;
use irc::proto::command::{CapSubCommand, Command};
use rand::prelude::*;
use regex::Regex;
use std::collections::HashMap;
use std::sync::mpsc::channel;
use std::sync::{Arc};
use std::{cmp, convert::TryInto};

pub struct CommandHash {
    pub commands: HashMap<String, ChatCommand>,
}

impl CommandHash {
    pub fn new() -> CommandHash {
        let commands = HashMap::<String, ChatCommand>::new();
        CommandHash { commands: commands }
    }
    pub fn add_command(&mut self, new_command: ChatCommand, command_symbol: String) {
        //self.commands.get_mut().unwrap().insert(format!("{}{}", command_symbol, new_command.command), new_command);
        self.commands.insert(
            format!("{}{}", command_symbol, new_command.command),
            new_command,
        );
    }
}

pub struct Arabot {
    pub name: String,
    oauth: String,
    pub twitch_channel: String,
    pub incoming_queue: Vec<ChatMessage>,
    pub answer_queue: Vec<Reply>,
    //pub commands: Mutex<HashMap<String, ChatCommand<F>>>,
    pub command_symbol: String,
    message_wait: u64,
}

impl Arabot {
    pub fn new(
        name: String,
        oauth: String,
        twitch_channel: String,
        command_symbol: String,
        message_wait: u64,
    ) -> Arabot {
        let m: Vec<ChatMessage> = Vec::new();
        let a: Vec<Reply> = Vec::new();
        let tc = String::from(&twitch_channel);
        let mut hash = String::from("#");
        hash.push_str(&tc);
        Arabot {
            name: name,
            oauth: oauth,
            twitch_channel: String::from(hash),
            incoming_queue: m,
            answer_queue: a,
            command_symbol: String::from(command_symbol),
            message_wait: message_wait,
        }
    }
    pub async fn start_bot(
        &self,
        commands: Box<CommandHash>,
        emote_list: Vec<String>,
        num_winners: usize,
        winner_message: String,
        perfect_guess_message: String,
        location_path: String
    ) -> Result<(), Error> {
        let mut commands = Box::new(commands);
        let ongoing_votes: HashMap<String, VoteObj> = HashMap::new();
        let regex_collection = VoteRegex::new();
        let irc_client_config = client::data::config::Config {
            nickname: Some(String::from(&self.name)),
            channels: vec![String::from(&self.twitch_channel)],
            password: Some(String::from(&self.oauth)),
            server: Some(String::from("irc.chat.twitch.tv")),
            port: Some(6697),
            use_tls: Some(true),
            ping_time: Some(300),
            ping_timeout: Some(300),
            ..client::data::config::Config::default()
        };

        let mut client = Client::from_config(irc_client_config).await?;
        client.identify()?;

        let (ms, mr) = channel::<client::prelude::Message>(); //message send and receive
        let (cs, cr) = channel::<ChatMessage>(); //command send and receive
        let (rs, rr) = channel::<(String, String)>(); //respond send and receive

        let thread_reg = Regex::new(r"badges=[a-zA-Z0-9/,_-]*;").unwrap();
        let message_thread = thread::spawn(move || {
            loop {
                let msg = mr.recv().unwrap();
                if let Command::PRIVMSG(channel, message) = &msg.command {
                    //                  chat_message.text = String::from(msg);
                    let match_string = msg.to_string();
                    let badge_match = thread_reg.find(&match_string).unwrap().as_str();

                    let el: Elevation = if badge_match.contains("broadcaster") {
                        Elevation::Broadcaster
                    } else if badge_match.contains("moderator") {
                        Elevation::Moderator
                    } else {
                        Elevation::Viewer
                    };
                    let chat_message = ChatMessage {
                        user: String::from(msg.source_nickname().unwrap_or("No username found")),
                        roles: el,
                        text: String::from(message),
                        channel: String::from(channel),
                    };
                    println!("{}: {}", chat_message.user, chat_message.text);
                    cs.send(chat_message).unwrap();
                }
            }
        });

        let arabot_symbol = Arc::new(String::from(self.command_symbol.as_str()));
        let cloned_arabot_symbol = Arc::clone(&arabot_symbol);
        let repeat_channel = self.twitch_channel.clone();
        let command_thread = thread::spawn(move || {
            //create list of commands with automatic updates, spawn a thread per that will sleep
            //until it's time to update
            for (_, command) in &commands.commands {
                if command.repeat_interval > 0 {
                    let repeat_interval = command.repeat_interval;
                    let repeat_message = command.response_message.clone();
                    let repeat_channel_clone = repeat_channel.clone();
                    if !command.command.contains("svote"){
                        let rs_clone = rs.clone();
                        let _ = thread::spawn(move || {
                            loop {
                                rs_clone.send((format!("{}", repeat_message), repeat_channel_clone.to_owned())).unwrap();
                                thread::sleep(time::Duration::from_secs(repeat_interval));
                            }
                        });
                    }
                }
            }

            let mut command_reg = Regex::new(r"").unwrap();
            command_reg = Arabot::generate_regex(&commands.commands, &cloned_arabot_symbol);

            let mut votes = VoteObj::new(0, String::from(""));
            loop {
                //let locked_commands = cloned_commands.commands;
                let cmd = cr.recv().unwrap();
                if command_reg.is_match(&cmd.text.trim()) {
                    let command = command_reg.find(&cmd.text.trim()).unwrap().as_str();
                    //TODO: svote, evote, extend, remember, vote, add command
                    match &command[1..] {
                        //special commands are placed in their own patterns in the match, while "regular" commands all go into default.
                        "hello" => rs
                            .send((format!("Hello, {}", cmd.user), cmd.channel))
                            .unwrap(),
                        "svote" => {
                            if !*votes.has_started.lock().unwrap()
                                && regex_collection.time_regex.is_match(&cmd.text)
                            {
                                let time = convert_string_int(&String::from(
                                    regex_collection
                                        .time_regex
                                        .find(&cmd.text)
                                        .unwrap()
                                        .as_str(),
                                ));
                                let mut location = String::from("");
                                if regex_collection.file_regex.is_match(&cmd.text) {
                                    location = format!("{}{}", location_path, String::from(
                                        regex_collection
                                            .file_regex
                                            .captures(&cmd.text)
                                            .unwrap()
                                            .get(1)
                                            .unwrap()
                                            .as_str(),
                                    ));
                                }
                                let rs_clone = rs.clone();
                                votes = VoteObj::new(time, location);
                                let votes_clone = Arc::clone(&votes);
                                let repeat_message = commands.commands.get_mut(command).unwrap().response_message.clone();
                                let repeat_channel_clone = repeat_channel.clone();
                                let repeat_interval = commands.commands.get_mut(command).unwrap().repeat_interval;
                                let _ = thread::spawn(move || {
                                    loop {
                                        if *votes_clone.has_started.lock().unwrap(){
                                            rs_clone.send((format!("{}", repeat_message), repeat_channel_clone.to_owned())).unwrap();
                                        }
                                        thread::sleep(time::Duration::from_secs(repeat_interval));
                                    }
                                });
                                let votes_clone = Arc::clone(&votes);
                                let rs_clone = rs.clone();
                                *votes.active_thread.lock().unwrap() = thread::spawn(move || {
                                    rs_clone
                                        .send((
                                            String::from("Voting session started!"),
                                            String::from(cmd.channel.as_str()),
                                        ))
                                        .unwrap();
                                    VoteObj::start_vote(votes_clone);
                                    rs_clone
                                        .send((
                                            String::from("Voting session ended!"),
                                            String::from(cmd.channel.as_str()),
                                        ))
                                        .unwrap();
                                });
                            }
                        }
                        "evote" => {
                            while *votes.has_started.lock().unwrap() {
                                votes.active_thread.lock().unwrap().thread().unpark();
                            }
                        }
                        "extend" => {
                            if regex_collection.time_regex.is_match(&cmd.text) {
                                let mut response = "";
                                votes.time_left.lock().unwrap().push(convert_string_int(
                                    &String::from(
                                        regex_collection
                                            .time_regex
                                            .find(&cmd.text)
                                            .unwrap()
                                            .as_str(),
                                    ),
                                ));
                                if !*votes.has_started.lock().unwrap() {
                                    response = "The voting session has been reopened!";
                                    let votes_clone = Arc::clone(&votes);
                                    let rs_clone = rs.clone();
                                    rs.send((
                                        String::from(response),
                                        String::from(cmd.channel.as_str()),
                                    ))
                                    .unwrap();
                                    *votes.active_thread.lock().unwrap() =
                                        thread::spawn(move || {
                                            VoteObj::start_vote(votes_clone);
                                            rs_clone
                                                .send((
                                                    String::from("Voting session ended!"),
                                                    String::from(cmd.channel.as_str()),
                                                ))
                                                .unwrap();
                                        });
                                } else {
                                    response = "The voting session has been extended!";
                                    rs.send((
                                        String::from(response),
                                        String::from(cmd.channel.as_str()),
                                    ))
                                    .unwrap();
                                }
                            }
                        }
                        "remember" => {}
                        "result" => {
                            //TODO: replace 3 with variable from config file.
                            if regex_collection.time_regex.is_match(&cmd.text) {
                                let winning_time = convert_string_int(&regex_collection.time_regex.find(&cmd.text).unwrap().as_str().to_string());
                                let num_show = cmp::min(votes.times.lock().unwrap().len(), num_winners);
                                let mut time_vector: Vec<(u64, String, String)> = Vec::new();

                                for (username, time) in &*votes.times.lock().unwrap() {
                                    time_vector
                                        .push((convert_string_int(time)-winning_time, String::from(time), String::from(username)));
                                }
                                if time_vector.len() == 0 {
                                    continue;
                                }

                                time_vector.sort_by_key(|time| time.0);
                                let winning_message = if time_vector[0].0 == winning_time {
                                    perfect_guess_message.clone()
                                } else {
                                    winner_message.clone()
                                };
                                rs.send((
                                    format!("{} {}", time_vector[0].2, winner_message),
                                    String::from(cmd.channel.as_str()),
                                ))
                                .unwrap();

                                for i in 0..num_show {
                                    rs.send((
                                        format!(
                                            "{}) {} {}",
                                            i + 1,
                                            time_vector[i].2,
                                            time_vector[i].1
                                        ),
                                        String::from(cmd.channel.as_str()),
                                    ))
                                    .unwrap();
                                }
                            }
                        }
                        "myvote" => {
                            if votes.times.lock().unwrap().contains_key(&cmd.user) {
                                rs.send((
                                    format!(
                                        "{} voted on {}",
                                        &cmd.user,
                                        String::from(&votes.times.lock().unwrap()[&cmd.user])
                                    ),
                                    String::from(cmd.channel.as_str()),
                                ))
                                .unwrap();
                            } else {
                                rs.send((
                                    format!("{}, you have not voted in this session", &cmd.user),
                                    String::from(cmd.channel.as_str()),
                                ))
                                .unwrap();
                            }
                        }
                        "add" => {} //for adding new commands
                        "vote" => {
                            //TODO: add the use of the hashmap with votes
                            //TODO: add regex to recognize where to put the vote
                            //votes.has_started = true; //TODO: remove, only here for testing purposes
                            if *votes.has_started.lock().unwrap() {
                                if regex_collection.time_regex.is_match(&cmd.text) {
                                    let time = regex_collection
                                        .time_regex
                                        .find(&cmd.text)
                                        .unwrap()
                                        .as_str();
                                    votes.add_vote(
                                        String::from(cmd.user.as_str()),
                                        String::from(time.trim()),
                                    );
                                    rs.send((
                                        format!("{} voted on {}", cmd.user, time),
                                        cmd.channel,
                                    ))
                                    .unwrap();
                                } else {
                                    rs.send((
                                        format!(
                                            "@{} {}",
                                            cmd.user,
                                            commands.commands.get_mut(command).unwrap().help
                                        ),
                                        cmd.channel,
                                    ))
                                    .unwrap();
                                }
                            } else {
                                rs.send((
                                    format!(
                                        "@{} there is currently no active voting session",
                                        cmd.user
                                    ),
                                    cmd.channel,
                                ))
                                .unwrap()
                            }
                        }
                        "help" => {
                            if !regex_collection.help_regex.is_match(&cmd.text) {
                                let mut viewer_commands = String::from("Normal commands: ");
                                let mut moderator_commands =
                                    String::from("Moderator only commands: ");
                                let mut broadcaster_commands =
                                    String::from("Broadcaster only commands: ");

                                for (k, c) in &commands.commands {
                                    match c.elevation {
                                        Elevation::Viewer => {
                                            viewer_commands.push_str(&format!("{}, ", &k[1..]))
                                        }
                                        Elevation::Moderator => {
                                            moderator_commands.push_str(&format!("{}, ", &k[1..]))
                                        }
                                        Elevation::Broadcaster => {
                                            broadcaster_commands.push_str(&format!("{}, ", &k[1..]))
                                        }
                                    }
                                }
                                rs.send((
                                    format!(
                                        "{} {} {} ",
                                        viewer_commands, moderator_commands, broadcaster_commands
                                    ),
                                    cmd.channel,
                                ))
                                .unwrap();
                            } else {
                                let help_command = format!(
                                    "{}{}",
                                    &cloned_arabot_symbol,
                                    regex_collection
                                        .help_regex
                                        .captures(&cmd.text)
                                        .unwrap()
                                        .get(1)
                                        .unwrap()
                                        .as_str()
                                );
                                if command_reg.is_match(&help_command) {
                                    rs.send((
                                        format!(
                                            "{}",
                                            commands
                                                .commands
                                                .get_mut(help_command.as_str())
                                                .unwrap()
                                                .help
                                        ),
                                        cmd.channel,
                                    ))
                                    .unwrap();
                                }
                            }
                        }
                        "slots" => {
                            let emote1 = emote_list.choose(&mut rand::thread_rng()).unwrap();
                            let emote2 = emote_list.choose(&mut rand::thread_rng()).unwrap();
                            let emote3 = emote_list.choose(&mut rand::thread_rng()).unwrap();

                            if emote1.as_str() == emote2.as_str()
                                && emote2.as_str() == emote3.as_str()
                            {
                                rs.send((
                                    format!(
                                        "{} {} {} JACKPOT @{}",
                                        emote1, emote2, emote3, cmd.user
                                    ),
                                    cmd.channel,
                                ))
                                .unwrap();
                            } else {
                                rs.send((
                                    format!("{} {} {} @{}", emote1, emote2, emote3, cmd.user),
                                    cmd.channel,
                                ))
                                .unwrap();
                            }
                        }
                        _default => rs
                            .send((
                                format!(
                                    "{}",
                                    (commands.commands.get_mut(command).unwrap().response)(
                                        String::from(&cmd.user),
                                        cmd.text
                                    )
                                ),
                                String::from(&cmd.channel),
                            ))
                            .unwrap(), //rs.send((format!("Error occured: No command found"), cmd.channel)).unwrap(),
                    }
                }
                //if cmd.text.contains("!hello"){
                //
                //}
            }
        });

        let mut stream = client.stream()?;
        client
            .send(Command::CAP(
                None,
                CapSubCommand::REQ,
                Some(String::from("twitch.tv/tags")),
                None,
            ))
            .unwrap();

        let client = Arc::new(client);
        let cloned_client = Arc::clone(&client);
        let tmp_wait = self.message_wait;
        let answer_thread = thread::spawn(move || loop {
            let (response, channel) = rr.recv().unwrap();
            cloned_client.send_privmsg(&channel, &response).unwrap();
            thread::sleep(time::Duration::from_millis(tmp_wait));
        });

        println!("Connected to {}", self.twitch_channel);

        while let Some(message) = stream.next().await.transpose()? {
            ms.send(message.clone()).unwrap();
        }
        let _ = message_thread.join();
        let _ = command_thread.join();
        let _ = answer_thread.join();
        Ok(())
    }
    fn generate_regex(commands: &HashMap<String, ChatCommand>, command_symbol: &String) -> Regex {
        let mut command_reg: String = String::from(format!(r"^"));
        for i in commands.keys() {
            //        command_reg.push(format!(r"{}{}", command_symbol, commands[i].command))
            command_reg
                .push_str(format!(r"^{}{}\b|", command_symbol, commands[i].command).as_str());
        }
        command_reg.push_str(format!(r"^{}hello\b", command_symbol).as_str());
        command_reg.push_str(r"{1}");
        regex::Regex::new(&command_reg).unwrap()
    }
}
fn convert_string_int(time: &String) -> u64 {
    let working_string = time.trim().split(":");
    let time_vec: Vec<&str> = working_string
        .collect::<Vec<&str>>()
        .into_iter()
        .rev()
        .collect();
    let mut time_seconds = 0;
    for i in 0..time_vec.len() {
        let local_time: u64 = time_vec[i].parse().unwrap();
        let pow_val: u32 = i.try_into().unwrap();
        time_seconds = time_seconds + local_time * (60u64.pow(pow_val));
    }
    time_seconds
}
