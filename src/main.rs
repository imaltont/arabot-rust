mod config;

//use std::io;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use arabot::arabot::message::{ChatCommand, Elevation};
use arabot::arabot::{Arabot, CommandHash};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bot = Arabot::new(
        (&config::CONFIG.name).to_string(),
        (&config::CONFIG.oauth).to_string(),
        "imaltont".to_string(),
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

    let hello = ChatCommand::new(
        String::from("hello"),
        Elevation::Viewer,
        Box::new(|_user, _text| String::from("Hello")),
        String::from("Says hello"),
        0,
    );
    let sluts = ChatCommand::new(
        String::from("sluts"),
        Elevation::Viewer,
        Box::new(|_user, _text| {
            String::from("SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm")
        }),
        String::from("Many SkopsGasm"),
        0,
    );
    let bee = ChatCommand::new(
        String::from("bee"),
        Elevation::Viewer,
        Box::new(|_user, _text| String::from("Thex will do fridge % in ∞ days")),
        String::from("Gives random bee facts"),
        0,
    );
    let vote = ChatCommand::new(
        String::from("vote"),
        Elevation::Viewer,
        Box::new(|_user, _text| String::from("Registers a vote")),
        String::from(
            "Lets you register a vote while one is active. You can vote with h:mm:ss or a number",
        ),
        0,
    );
    let help = ChatCommand::new(
        String::from("help"),
        Elevation::Viewer,
        Box::new(|_user, _text| String::from("Diaplays all available commands")),
        String::from("Displays all available commands, or specific use of a command if supplied."),
        0,
    );
    let slots = ChatCommand::new(
        String::from("slots"),
        Elevation::Viewer,
        Box::new(|_user, _text| String::from("test")),
        String::from("Rolls random emotes with a possibility to get a jackpot"),
        0,
    );
    let specs = ChatCommand::new(
        String::from("specs"),
        Elevation::Viewer,
        Box::new(|_user, _text| String::from("I dno, check the description.")),
        String::from("Lists the specs"),
        0,
    );
    commands.add_command(hello, String::from(bot.command_symbol.as_str()));
    commands.add_command(sluts, String::from(bot.command_symbol.as_str()));
    commands.add_command(slots, String::from(bot.command_symbol.as_str()));
    commands.add_command(vote, String::from(bot.command_symbol.as_str()));
    commands.add_command(help, String::from(bot.command_symbol.as_str()));
    commands.add_command(bee, String::from(bot.command_symbol.as_str()));
    commands.add_command(specs, String::from(bot.command_symbol.as_str()));
    let w = bot.start_bot(commands, emotes);
    w.await.unwrap();
    Ok(())
}
