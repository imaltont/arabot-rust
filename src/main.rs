mod config;

//use std::io;
use tokio::prelude::*;
use std::fs::File;
use std::io::{self,BufReader};
use std::io::prelude::*;

use arabot::arabot::{Arabot, CommandHash};
use arabot::arabot::message::{ChatCommand, Elevation};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let bot = Arabot::new((&config::CONFIG.name).to_string(), (&config::CONFIG.oauth).to_string(), "imaltont".to_string(), "!".to_string(), 1);

    //Runtime::new().expect("Error").block_on(bot.start_bot());
    let mut commands = Box::new(CommandHash::new());
    let mut emotes: Vec<String> = Vec::new();
    let emote_file = File::open("emotes.txt")?;
    let reader = BufReader::new(emote_file);
    for line in reader.lines() {
        emotes.push(line.unwrap());
    }

    let sluts = ChatCommand::new(String::from("sluts"), Elevation::Viewer, Box::new(|_user, _text| String::from("SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm")), String::from("SkopsGasme"), 0);
    let slots = ChatCommand::new(String::from("slots"), Elevation::Viewer, Box::new(|_user, _text|
            String::from("test")), String::from("Rolls random emotes with a possibility to get a
            jackpot"), 0);
    commands.add_command(sluts, String::from(bot.command_symbol.as_str()));
    commands.add_command(slots, String::from(bot.command_symbol.as_str()));
    let w = bot.start_bot(commands, emotes);
    w.await.unwrap();
    Ok(())
}
