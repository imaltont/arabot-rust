mod config;

//use std::io;
use serde::{Deserialize, Serialize};
//use serde_json::Result;

use std::fs::{self};
use std::{path::Path};

use arabot::arabot::message::{ChatCommand, Elevation};
use arabot::arabot::{Arabot, CommandHash};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let settings = read_settings();

    let command_list = read_commands(&settings.command_location);

    let delay: u64 = match settings.message_delay{
        Elevation::Moderator => 300,
        _default => 1500
    };

    let bot = Arabot::new(
        (&config::CONFIG.name).to_string(),
        (&config::CONFIG.oauth).to_string(),
        settings.channel,
        settings.command_prefix,
        delay
    );

    //Runtime::new().expect("Error").block_on(bot.start_bot());
    
    let mut commands = Box::new(CommandHash::new());

    for command in command_list {
        let response = String::from(command.response);
        let com = ChatCommand::new(
            command.name.clone(),
            command.elevation.clone(),
            command.reminder_message.clone(),
            Box::new(move |_user, _text| response.to_owned()),
            command.help.clone(),
            command.reminder_time
        );
        commands.add_command(com, String::from(bot.command_symbol.as_str()));
    }

    let w = bot.start_bot(commands, settings.slots_emotes, settings.number_of_results_shown, settings.winner_message, settings.perfect_guess_message);
    w.await.unwrap();
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct Settings {
    channel: String,
    command_prefix: String,
    message_delay: Elevation,
    winner_message: String,
    perfect_guess_message: String,
    number_of_results_shown: usize,
    vote_save_location: String,
    command_location: String,
    slots_emotes: Vec<String>,
}
#[derive(Serialize, Deserialize)]
struct Commands {
    commands: Vec<Command>
}
#[derive(Serialize, Deserialize)]
struct  Command{
    name: String,
    active: bool,
    elevation: Elevation,
    reminder_message: String,
    response: String,
    help: String,
    reminder_time: u64,
}
fn read_settings() -> Settings {
    let path = Path::new("./settings/settings.json");
    let data = fs::read_to_string(path).expect("Unable to read file");
    let settings: Settings = serde_json::from_str(&data).expect("Unable to parse");
    settings
}
fn read_commands(file_path: &String) -> Vec<Command> {
    let path = Path::new(file_path);
    let data = fs::read_to_string(path).expect("Unable to read file");
    let commands: Commands = serde_json::from_str(&data).expect("Unable to parse");
    commands.commands
}
