use std::{env, error, thread, time};
use message::{ChatMessage,Reply,ChatCommand, Elevation};
mod message;

use std::sync::mpsc::{channel};
use std::sync::Arc;
use regex::{Regex, RegexSet};
use irc::client::prelude::*;
use futures::prelude::*;
use irc::client;
use irc::error::Error;
use irc::proto::command::{CapSubCommand,Command};


pub struct Arabot {
    pub name: String,
    oauth: String,
    pub twitch_channel: String,
    pub incoming_queue: Vec<ChatMessage>,
    pub answer_queue: Vec<Reply>,
    pub commands: Vec<ChatCommand>,
    command_symbol: String,
    message_wait: u64
}

impl Arabot{
    pub fn new(name: String, oauth: String, twitch_channel: String, command_symbol: String, message_wait: u64) -> Arabot {

        let mut m: Vec<ChatMessage> = Vec::new();
        let mut a: Vec<Reply> = Vec::new();
        let mut c: Vec<ChatCommand> = Vec::new();
        let tc = String::from(&twitch_channel);
        let mut hash = String::from("#");
        hash.push_str(&tc);
        Arabot{name: name, oauth: oauth, twitch_channel: String::from(hash), incoming_queue: m, answer_queue: a, commands: c, command_symbol: String::from(command_symbol), message_wait: message_wait}
    }
    pub fn clone (old_bot: &Arabot) -> Arabot {
        let name = String::from(&old_bot.name);
        let oauth = String::from(&old_bot.oauth);
        let command_symbol = String::from(&old_bot.command_symbol);
        let message_wait = old_bot.message_wait;
        let mut m: Vec<ChatMessage> = Vec::new();
        let mut a: Vec<Reply> = Vec::new();
        let mut c: Vec<ChatCommand> = Vec::new();
        for i in 0..old_bot.commands.len() {
            c.push(ChatCommand{command: String::from(&old_bot.commands[i].command), elevation: old_bot.commands[i].elevation, response: old_bot.commands[i].response, help: String::from(&old_bot.commands[i].help), repeat_interval: old_bot.commands[i].repeat_interval});
        }
        let tc = String::from(&old_bot.twitch_channel);
        Arabot{name: name, oauth: oauth, twitch_channel: tc, incoming_queue: m, answer_queue: a, commands: c, command_symbol: String::from(command_symbol), message_wait: message_wait}
    }
    pub async fn start_bot(&self)-> Result<(), Error>{
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

        let arabot = Arc::new(Arabot::clone(&self));
        let cloned_arabot = Arc::clone(&arabot);
        let command_thread = thread::spawn(move || {
            let mut command_reg = generate_regex(&cloned_arabot.commands, &cloned_arabot.command_symbol);
            loop {
                let cmd = cr.recv().unwrap();
                //TODO insert proper logic for handling more than just !hello
                if command_reg.is_match(&cmd.text){
                    let command = command_reg.find(&cmd.text).unwrap().as_str();
                    println!("{}", command);
                    match command{
                        "!hello" => rs.send((format!("Hello, {}", cmd.user), cmd.channel)).unwrap(),
                        _default => rs.send((format!("Error occured: No command found"), cmd.channel)).unwrap(),
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

}

fn generate_regex(commands: &Vec<ChatCommand>, command_symbol: &String) -> Regex{
    let mut command_reg: String  = String::from(format!(r"^"));
    for i in 0..commands.len() {
//        command_reg.push(format!(r"{}{}", command_symbol, commands[i].command))
        command_reg.push_str(format!(r"{}{}|", command_symbol, commands[i].command).as_str());
    }
    command_reg.push_str(format!(r"{}hello", command_symbol).as_str());
    command_reg.push_str(r"{1}");
    regex::Regex::new(&command_reg).unwrap()
}
