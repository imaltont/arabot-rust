mod config;

use std::io;
use tokio::prelude::*;

use arabot::arabot::{Arabot, CommandHash};
use arabot::arabot::message::{ChatCommand, Elevation};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let bot = Arabot::new((&config::CONFIG.name).to_string(), (&config::CONFIG.oauth).to_string(), "imaltont".to_string(), "!".to_string(), 1);

    //Runtime::new().expect("Error").block_on(bot.start_bot());
    let mut commands = Box::new(CommandHash::new());
    let command = ChatCommand::new(String::from("sluts"), Elevation::Viewer, Box::new(|_user, _text| String::from("SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm")), String::from("SkopsGasme"), 0);
    commands.add_command(command, String::from(bot.command_symbol.as_str()));
    let w = bot.start_bot(commands);
    w.await.unwrap();
    Ok(())
}
