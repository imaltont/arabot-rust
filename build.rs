use config_struct::{Error, StructOptions};

fn main() -> Result<(), Error> {
    config_struct::create_struct("twitch_settings.json", "src/config.rs", &StructOptions::default())
}
