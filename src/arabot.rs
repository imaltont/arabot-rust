use std::{env, error, thread, time};
use message::{ChatMessage,Reply,ChatCommand, Elevation};
mod message;

use std::sync::mpsc::{channel};
use std::sync::{Mutex,Arc};
use std::collections::HashMap;
use regex::Regex;
use irc::client::prelude::*;
use futures::prelude::*;
use irc::client;
use irc::error::Error;
use irc::proto::command::{CapSubCommand,Command};


pub struct Arabot<F: FnMut(String, String) -> String + 'static> {
    pub name: String,
    oauth: String,
    pub twitch_channel: String,
    pub incoming_queue: Vec<ChatMessage>,
    pub answer_queue: Vec<Reply>,
    pub commands: Mutex<HashMap<String, ChatCommand<F>>>,
    command_symbol: String,
    message_wait: u64
}

impl<F: FnMut(String, String) -> String + 'static> Arabot<F>{
    pub fn new(name: String, oauth: String, twitch_channel: String, command_symbol: String, message_wait: u64) -> Arabot<F> {

        let mut m: Vec<ChatMessage> = Vec::new();
        let mut a: Vec<Reply> = Vec::new();
        let mut c: Mutex<HashMap<String, ChatCommand<F>>> = Mutex::new(HashMap::new());
        let tc = String::from(&twitch_channel);
        let mut hash = String::from("#");
        hash.push_str(&tc);
        Arabot{name: name, oauth: oauth, twitch_channel: String::from(hash), incoming_queue: m, answer_queue: a, commands: c, command_symbol: String::from(command_symbol), message_wait: message_wait}
    }
    pub fn from (old_bot: &Arabot<F>) -> Arabot<F> {
        let name = String::from(&old_bot.name);
        let oauth = String::from(&old_bot.oauth);
        let command_symbol = String::from(&old_bot.command_symbol);
        let message_wait = old_bot.message_wait;
        let mut m: Vec<ChatMessage> = Vec::new();
        let mut a: Vec<Reply> = Vec::new();
        let mut c: Mutex<HashMap<String, ChatCommand<F>>> = Mutex::new(HashMap::new());
        let mut tmp_lock = old_bot.commands.lock().unwrap();
        for i in tmp_lock.keys() {
            c.get_mut().unwrap().insert(format!("{}{}", old_bot.command_symbol, tmp_lock[i].command),ChatCommand{command: String::from(&tmp_lock[i].command), elevation: tmp_lock[i].elevation, response: Arc::clone(&tmp_lock[i].response), help: String::from(&tmp_lock[i].help), repeat_interval: tmp_lock[i].repeat_interval});
        }
        let tc = String::from(&old_bot.twitch_channel);
        Arabot{name: name, oauth: oauth, twitch_channel: tc, incoming_queue: m, answer_queue: a, commands: c, command_symbol: String::from(command_symbol), message_wait: message_wait}
    }
    pub fn add_command(&mut self, new_command: ChatCommand<F>){
        self.commands.get_mut().unwrap().insert(format!("{}{}", self.command_symbol, new_command.command), new_command);
    }
    pub async fn start_bot(&mut self)-> Result<(), Error>{
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
                if let Command::PRIVMSG(channel, message) = &msg.command{
//                  chat_message.text = String::from(msg);
                    let match_string = msg.to_string();
                    let badge_match = thread_reg.find(&match_string).unwrap().as_str();

                    let el: Elevation = if badge_match.contains("broadcaster") {
                        Elevation::Broadcaster
                    } else if badge_match.contains("moderator"){
                        Elevation::Moderator
                    } else {
                        Elevation::Viewer
                    };
                    let chat_message = ChatMessage{user: String::from(msg.source_nickname().unwrap_or("No username found")), roles: el, text: String::from(message), channel: String::from(channel)};
                    println!("{}: {}", chat_message.user, chat_message.text);
                    cs.send(chat_message).unwrap();
                }
            }
        });

        let arabot = Arc::new(Arabot::from(self));
        let cloned_arabot = Arc::clone(&arabot);
        let command_thread = thread::spawn(move || {
            let commands = cloned_arabot.commands.lock().unwrap();
            let mut command_reg = Arabot::generate_regex(&commands, &cloned_arabot.command_symbol);
            loop {
                let cmd = cr.recv().unwrap();
                //TODO insert proper logic for handling more than just !hello
                if command_reg.is_match(&cmd.text){
                    let command = command_reg.find(&cmd.text).unwrap().as_str();
                    match command{ //special commands are placed in their own patterns in the match, while "regular" commands all go into default.
                        "!hello" => rs.send((format!("Hello, {}", cmd.user), cmd.channel)).unwrap(),
                        _default => rs.send((format!("{}", (commands[command].response)(String::from(&cmd.user), cmd.text)), String::from(&cmd.user))).unwrap(),//rs.send((format!("Error occured: No command found"), cmd.channel)).unwrap(),
                    }
                }
                //if cmd.text.contains("!hello"){
                //   
                //}
            }
        });


        let mut stream = client.stream()?;
        client.send(Command::CAP(None, CapSubCommand::REQ, Some(String::from("twitch.tv/tags")), None)).unwrap(); 

        let client = Arc::new(client);
        let cloned_client = Arc::clone(&client);
        let tmp_wait = self.message_wait;
        let answer_thread = thread::spawn(move || {
            loop{
                let (response, channel) = rr.recv().unwrap();
                cloned_client.send_privmsg(&channel, &response).unwrap();
                thread::sleep(time::Duration::from_millis(tmp_wait));
            }

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
    fn generate_regex(commands: &HashMap<String, ChatCommand<F>>, command_symbol: &String) -> Regex{
        let mut command_reg: String  = String::from(format!(r"^"));
        for i in commands.keys() {
    //        command_reg.push(format!(r"{}{}", command_symbol, commands[i].command))
            command_reg.push_str(format!(r"{}{}|", command_symbol, commands[i].command).as_str());
        }
        command_reg.push_str(format!(r"{}hello", command_symbol).as_str());
        command_reg.push_str(r"{1}");
        regex::Regex::new(&command_reg).unwrap()
    }

}

