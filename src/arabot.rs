use std::{env, error};
use message::{ChatMessage,Reply,ChatCommand, Elevation};
mod message;

use std::sync::mpsc::{channel};
use std::sync::Arc;
use std::thread;
use regex::Regex;
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
    pub commands: Vec<ChatCommand>
}

impl Arabot{
    pub fn new(name: String, oauth: String, twitch_channel: String) -> Arabot {

        let mut m: Vec<ChatMessage> = Vec::new();
        let mut a: Vec<Reply> = Vec::new();
        let mut c: Vec<ChatCommand> = Vec::new();
        let tc = String::from(&twitch_channel);
        let mut hash = String::from("#");
        hash.push_str(&tc);
        Arabot{name: name, oauth: oauth, twitch_channel: String::from(hash), incoming_queue: m, answer_queue: a, commands: c}
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

        let command_thread = thread::spawn(move || {
            loop {
                let cmd = cr.recv().unwrap();
                //TODO insert proper logic for handling more than just !hello
                if cmd.text.contains("!hello"){
                    rs.send((format!("Hello, {}", cmd.user), cmd.channel)).unwrap();
                }
            }
        });


        let mut stream = client.stream()?;
        client.send(Command::CAP(None, CapSubCommand::REQ, Some(String::from("twitch.tv/tags")), None)).unwrap(); 

        let client = Arc::new(client);
        let cloned_client = Arc::clone(&client);
        let answer_thread = thread::spawn(move || {
            //TODO add sleep so not banned for spam
            loop{
                let (response, channel) = rr.recv().unwrap();
                cloned_client.send_privmsg(&channel, &response).unwrap();
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
