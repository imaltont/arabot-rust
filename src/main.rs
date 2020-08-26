mod config;

//use std::io;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use tokio::prelude::*;

use arabot::arabot::message::{ChatCommand, Elevation};
use arabot::arabot::{Arabot, CommandHash};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bot = Arabot::new(
        (&config::CONFIG.name).to_string(),
        (&config::CONFIG.oauth).to_string(),
        "thextera_".to_string(),
        "!".to_string(),
        1,
    );

    //Runtime::new().expect("Error").block_on(bot.start_bot());
    let mut commands = Box::new(CommandHash::new());
    let mut emotes: Vec<String> = Vec::new();
    let emote_file = File::open("emotes.txt")?;
    let reader = BufReader::new(emote_file);
    for line in reader.lines() {
        emotes.push(line.unwrap());
    }

    let sluts = ChatCommand::new(
        String::from("sluts"),
        Elevation::Viewer,
        Box::new(|_user, _text| {
            String::from("SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm")
        }),
        String::from("SkopsGasme"),
        0,
    );
    let bee = ChatCommand::new(
        String::from("bee"),
        Elevation::Viewer,
        Box::new(|_user, _text| String::from("Thex will do fridge % in ∞ days")),
        String::from("SkopsGasme"),
        0,
    );
    let vote = ChatCommand::new(
        String::from("vote"),
        Elevation::Viewer,
        Box::new(|_user, _text| String::from("Registers a vote")),
        String::from("Lets you register a vote while active. You can vote with h:mm:ss"),
        0,
    );
    let slots = ChatCommand::new(
        String::from("slots"),
        Elevation::Viewer,
        Box::new(|_user, _text| String::from("test")),
        String::from(
            "Rolls random emotes with a possibility to get a
            jackpot",
        ),
        0,
    );
    commands.add_command(sluts, String::from(bot.command_symbol.as_str()));
    commands.add_command(slots, String::from(bot.command_symbol.as_str()));
    commands.add_command(vote, String::from(bot.command_symbol.as_str()));
    commands.add_command(bee, String::from(bot.command_symbol.as_str()));
    let w = bot.start_bot(commands, emotes);
    w.await.unwrap();
    Ok(())
}
