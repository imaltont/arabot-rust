mod config;

use std::io;
use tokio::prelude::*;

use arabot::arabot::Arabot;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let bot = Arabot::new((&config::CONFIG.name).to_string(), (&config::CONFIG.oauth).to_string(), "imaltont".to_string());

    //Runtime::new().expect("Error").block_on(bot.start_bot());
    let w = bot.start_bot();
    w.await.unwrap();
    Ok(())
}
