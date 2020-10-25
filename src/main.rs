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
    let emote_file = File::open("emotes.txt")?; let reader = BufReader::new(emote_file);
    for line in reader.lines() {
        emotes.push(line.unwrap());
    }

    let hello = ChatCommand::new(
        String::from("hello"),
        Elevation::Viewer,
        String::from("Hello"),
        Box::new(|_user, _text| String::from("Hello")),
        String::from("Says hello"),
        0,
    );
    let sluts = ChatCommand::new(
        String::from("sluts"),
        Elevation::Viewer,
        String::from("SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm"),
        Box::new(|_user, _text| {
            String::from("SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm SkopsGasm")
        }),
        String::from("Many SkopsGasm"),
        0,
    );
    let vote = ChatCommand::new(
        String::from("vote"),
        Elevation::Viewer,
        String::from("Registers a vote"),
        Box::new(|_user, _text| String::from("Registers a vote")),
        String::from(
            "Lets you register a vote while one is active. You can vote with h:mm:ss or a number",
        ),
        0,
    );
    let myvote = ChatCommand::new(
        String::from("myvote"),
        Elevation::Viewer,
        String::from("Returns what you voted for"),
        Box::new(|_user, _text| String::from("Returns what you voted for")),
        String::from("Tells you the time or number you have voted for."),
        0,
    );
    let svote = ChatCommand::new(
        String::from("svote"),
        Elevation::Moderator,
        String::from("Starts a vote"),
        Box::new(|_user, _text| String::from("Starts a vote")),
        String::from(
            "Starts a new voting session. You can start it with a name if several are already running, the time in seconds or hour, minutes and/or seconds, and you can give it a name for saving the votes and results in a csv file",
        ),
        0,
    );
    let extend = ChatCommand::new(
        String::from("extend"),
        Elevation::Moderator,
        String::from("Extends the available time for a voting session"),
        Box::new(|_user, _text| String::from("Extends the available time for a voting session")),
        String::from(
            "Extends the remaining time of a voting session. You can extend it with a name if several are already running, the time in seconds or hour, minutes and/or seconds",
        ),
        0,
    );
    let evote = ChatCommand::new(
        String::from("evote"),
        Elevation::Moderator,
        String::from("Ends a vote"),
        Box::new(|_user, _text| String::from("Ends a vote")),
        String::from("Ends a voting session"),
        0,
    );
    let result = ChatCommand::new(
        String::from("result"),
        Elevation::Moderator,
        String::from("Registers the result of the voting session"),
        Box::new(|_user, _text| String::from("Registers the result of the voting session")),
        String::from(
            "Registers the result of the voting session. Register the result with h:mm:ss or a single number",
        ),
        0,
    );
    let help = ChatCommand::new(
        String::from("help"),
        Elevation::Viewer,
        String::from("Displays all available commands"),
        Box::new(|_user, _text| String::from("Displays all available commands")),
        String::from("Displays all available commands, or specific use of a command if supplied."),
        0,
    );
    let slots = ChatCommand::new(
        String::from("slots"),
        Elevation::Viewer,
        String::from("Rolls random emotes with a possibility to get a jackpot"),
        Box::new(|_user, _text| String::from("test")),
        String::from("Rolls random emotes with a possibility to get a jackpot"),
        0,
    );
    let specs = ChatCommand::new(
        String::from("specs"),
        Elevation::Viewer,
        String::from("I dno, check the description."),
        Box::new(|_user, _text| String::from("I dno, check the description.")),
        String::from("Lists the specs"),
        0,
    );
    let arttles = ChatCommand::new(
        String::from("arttles"),
        Elevation::Viewer,
        String::from("https://www.twitch.tv/thextera_/clip/MoralEndearingRavenNomNom"),
        Box::new(|_user, _text| String::from("https://www.twitch.tv/thextera_/clip/MoralEndearingRavenNomNom")),
        String::from("Lists the specs"),
        0,
    );
    commands.add_command(hello, String::from(bot.command_symbol.as_str()));
    commands.add_command(sluts, String::from(bot.command_symbol.as_str()));
    commands.add_command(slots, String::from(bot.command_symbol.as_str()));
    commands.add_command(vote, String::from(bot.command_symbol.as_str()));
    commands.add_command(myvote, String::from(bot.command_symbol.as_str()));
    commands.add_command(svote, String::from(bot.command_symbol.as_str()));
    commands.add_command(extend, String::from(bot.command_symbol.as_str()));
    commands.add_command(evote, String::from(bot.command_symbol.as_str()));
    commands.add_command(result, String::from(bot.command_symbol.as_str()));
    commands.add_command(help, String::from(bot.command_symbol.as_str()));
    commands.add_command(specs, String::from(bot.command_symbol.as_str()));
    commands.add_command(arttles, String::from(bot.command_symbol.as_str()));
    let w = bot.start_bot(commands, emotes);
    w.await.unwrap();
    Ok(())
}
