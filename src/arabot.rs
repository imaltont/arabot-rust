use std::{env, error};
use message::{Message,Reply};
mod message;

use irc::client::prelude::*;
use futures::prelude::*;
use irc::client;
use irc::error::Error;
use irc::proto::command::{CapSubCommand,Command};

pub struct Arabot {
    pub name: String,
    oauth: String,
    pub twitch_channel: String,
    pub incoming_queue: Vec<Message>,
    pub answer_queue: Vec<Reply>
}

impl Arabot{
    pub fn new(name: String, oauth: String, twitch_channel: String) -> Arabot {

        let mut m: Vec<Message> = Vec::new();
        let mut a: Vec<Reply> = Vec::new();
        let tc = String::from(&twitch_channel);
        let mut hash = String::from("#");
        hash.push_str(&tc);
        Arabot{name: name, oauth: oauth, twitch_channel: String::from(hash), incoming_queue: m, answer_queue: a}
    }
    pub async fn start_bot(&self)-> Result<(), Error>{
        let irc_client_config = client::data::config::Config {
            nickname: Some(String::from(&self.name)),
            channels: vec![String::from(&self.twitch_channel)],
            password: Some(String::from(&self.oauth)),
            server: Some(String::from("irc.chat.twitch.tv")),
            port: Some(6697),
            use_tls: Some(true),
            ..client::data::config::Config::default()
        };

        let mut client = Client::from_config(irc_client_config).await?;
        client.identify()?;

        let mut stream = client.stream()?;
        client.send(Command::CAP(None, CapSubCommand::REQ, Some(String::from("twitch.tv/tags")), None)).unwrap(); 
        while let Some(message) = stream.next().await.transpose()? {
            println!("{}", message);
            if let Command::PRIVMSG(channel, message) = message.command {
                if message.contains(&*client.current_nickname()) {
                    println!("{}", channel);
                    client.send_privmsg(&channel, "Hello").unwrap();
                }
            }
        }
        println!("test");
        Ok(())
    }
}
