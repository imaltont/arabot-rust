use message::{ChatCommand, ChatMessage, Elevation, Reply, VoteObj, VoteRegex};
use std::{thread, time};
pub mod message;

use futures::prelude::*;
use irc::client;
use irc::client::prelude::*;
use irc::error::Error;
use irc::proto::command::{CapSubCommand, Command};
use rand::prelude::*;
use regex::Regex;
use std::collections::HashMap;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};

pub struct CommandHash {
    pub commands: HashMap<String, ChatCommand>,
}

impl CommandHash {
    pub fn new() -> CommandHash {
        let mut commands = HashMap::<String, ChatCommand>::new();
        CommandHash { commands: commands }
    }
    pub fn add_command(&mut self, new_command: ChatCommand, command_symbol: String) {
        //self.commands.get_mut().unwrap().insert(format!("{}{}", command_symbol, new_command.command), new_command);
        self.commands.insert(
            format!("{}{}", command_symbol, new_command.command),
            new_command,
        );
    }
}

pub struct Arabot {
    pub name: String,
    oauth: String,
    pub twitch_channel: String,
    pub incoming_queue: Vec<ChatMessage>,
    pub answer_queue: Vec<Reply>,
    //pub commands: Mutex<HashMap<String, ChatCommand<F>>>,
    pub command_symbol: String,
    message_wait: u64,
}

impl Arabot {
    pub fn new(
        name: String,
        oauth: String,
        twitch_channel: String,
        command_symbol: String,
        message_wait: u64,
    ) -> Arabot {
        let mut m: Vec<ChatMessage> = Vec::new();
        let mut a: Vec<Reply> = Vec::new();
        let tc = String::from(&twitch_channel);
        let mut hash = String::from("#");
        hash.push_str(&tc);
        Arabot {
            name: name,
            oauth: oauth,
            twitch_channel: String::from(hash),
            incoming_queue: m,
            answer_queue: a,
            command_symbol: String::from(command_symbol),
            message_wait: message_wait,
        }
    }
    //    pub fn from (old_bot: &Arabot) -> Arabot {
    //        let name = String::from(&old_bot.name);
    //        let oauth = String::from(&old_bot.oauth);
    //        let command_symbol = String::from(&old_bot.command_symbol);
    //        let message_wait = old_bot.message_wait;
    //        let mut m: Vec<ChatMessage> = Vec::new();
    //        let mut a: Vec<Reply> = Vec::new();
    //        let tc = String::from(&old_bot.twitch_channel);
    //        Arabot{name: name, oauth: oauth, twitch_channel: tc, incoming_queue: m, answer_queue: a, command_symbol: String::from(command_symbol), message_wait: message_wait}
    //    }
    pub async fn start_bot(
        &self,
        commands: Box<CommandHash>,
        emote_list: Vec<String>,
    ) -> Result<(), Error> {
        let mut commands = Box::new(commands);
        let mut ongoing_votes: HashMap<String, VoteObj> = HashMap::new();
        let regex_collection = VoteRegex::new();
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
                if let Command::PRIVMSG(channel, message) = &msg.command {
                    //                  chat_message.text = String::from(msg);
                    let match_string = msg.to_string();
                    let badge_match = thread_reg.find(&match_string).unwrap().as_str();

                    let el: Elevation = if badge_match.contains("broadcaster") {
                        Elevation::Broadcaster
                    } else if badge_match.contains("moderator") {
                        Elevation::Moderator
                    } else {
                        Elevation::Viewer
                    };
                    let chat_message = ChatMessage {
                        user: String::from(msg.source_nickname().unwrap_or("No username found")),
                        roles: el,
                        text: String::from(message),
                        channel: String::from(channel),
                    };
                    println!("{}: {}", chat_message.user, chat_message.text);
                    cs.send(chat_message).unwrap();
                }
            }
        });

        let arabot_symbol = Arc::new(String::from(self.command_symbol.as_str()));
        let cloned_arabot_symbol = Arc::clone(&arabot_symbol);
        let command_thread = thread::spawn(move || {
            let mut command_reg = Regex::new(r"").unwrap();
            command_reg = Arabot::generate_regex(&commands.commands, &cloned_arabot_symbol);
            let mut bee_facts: [&str; 100] = [""; 100];
            //TODO: move this section to its own file/other place. Should read this in from a file
            //or have it as part of a script at some point.
            bee_facts[0] = "The practice of beekeeping dates back at least 4,500 years";
            bee_facts[1] =
                "Approximately one third of the food we eat is the result of honey bee pollination";
            bee_facts[2] = "In their 7-8 week lifespan, a worker bee will fly the equivalent distance of 1 ½ times the circumference of the Earth.";
            bee_facts[3] = "A productive queen can lay up to 3,500 eggs per day.";
            bee_facts[4] = "Mead, which is made from fermented honey, is the world’s oldest fermented beverage.";
            bee_facts[5] =
                "A single bee will produce only about 1/12 of a teaspoon of honey in its lifetime.";
            bee_facts[6] = "Bees are attracted by caffeine.";
            bee_facts[7] = "The perfect hexagons that form honeycomb hold the most amount of honey with the smallest amount of material (wax).";
            bee_facts[8] = "Honey bees are the only insect that produces food consumed by humans.";
            bee_facts[9] = "During a single collection trip, a honey bee will visit anywhere from 50 to 100 flowers.";
            bee_facts[10] =
                "Honey bees beat their wings 200 times per second, creating their trademark buzz.";
            bee_facts[11] = "Though bees have jointed legs, they do not possess anything like a kneecap, and therefore do not have knees.";
            bee_facts[12] =
                "There are three types of bees in every hive: a queen, worker bees, and drones.";
            bee_facts[13] = "Only drones are male.";
            bee_facts[14] =
                "In order to make a pound of honey, a hive of bees must fly 55,000 miles";
            bee_facts[15] = "Honey bees don’t sleep. Instead, they spend their nights motionless, conserving energy for the next day’s activities.";
            bee_facts[16] = "The honey bee is the official insect of Maine.";
            bee_facts[17] = "Honey was found in King Tut’s tomb, and, because it never spoils, it was still good!";
            bee_facts[18] = "Honey is the only known source of the antioxidant pinocembrin.";
            bee_facts[19] = "On average, Americans consume 1.31 pounds of honey every year.";
            bee_facts[20] = "A queen goes on what is called a “mating flight” where she leaves the hive and mates with anywhere from 5 to 45 different drones. She stores the sperm in her spermatheca, and has a lifetime supply, therefore only needing to take 1 mating flight in her lifetime.";
            bee_facts[21] = "Honey bees are not born knowing how to make honey. Instead, they are taught in the hive by older bees.";
            bee_facts[22] =
                "There are estimated to be nearly 212,000 beekeepers in the United States.";
            bee_facts[23] = "Honey is 25% sweeter than table sugar.";
            bee_facts[24] = "Honey is the only foodstuff that contains all of the necessary nutrients to sustain life.";
            bee_facts[25] = "Bee venom is used as a treatment for several ailments, including arthritis and high blood pressure.";
            bee_facts[26] = "Drones die after mating with a queen.";
            bee_facts[27] =
                "A single hive can produce anywhere from 60 to 100 pounds of honey every year.";
            bee_facts[28] = "The ancient Greeks and Romans viewed honey as a symbol of love, beauty, and fertility.";
            bee_facts[29] = "There are people in Africa that keep elephants out of their fields by keeping honey bee hives around the fields in what is called a “bee fence.”";
            bee_facts[30] = "In Greek mythology, Apollo is credited as being the first beekeeper.";
            bee_facts[31] = "Ancient peoples used to believe that bees were created from the carcasses of dead animals.";
            bee_facts[32] = "Bees were a very popular animal to include in Napoleonic heraldry.";
            bee_facts[33] = "In ancient Egypt, people paid their taxes with honey.";
            bee_facts[34] = "The ancient Greeks minted coins with bees on them.";
            bee_facts[35] = "Beeswax is found in many of our everyday products, including furniture polishes, cosmetics, and medicines.";
            bee_facts[36] = "The name ‘Melissa’ is derived from the Greek word for honey bee.";
            bee_facts[37] = "Beekeeping is said to be the second oldest professions.";
            bee_facts[38] = "Stone Age cave paintings have been found of ancient beekeepers. The oldest known art depicting honey gathering can be found in the Cave of the Spider near Valencia, Spain.";
            bee_facts[39] = "Every species of bee performs their communication dances differently.";
            bee_facts[40] =
                "The darker the honey, the greater amount of antioxidant properties it has.";
            bee_facts[41] = "Bees can be trained to locate buried land mines.";
            bee_facts[42] = "Ounce for ounce, honey bee venom is more deadly than cobra venom. Don’t worry, though – it takes 19 stings for every kilogram of a person’s body weight to be lethal.";
            bee_facts[43] = "The first Anglo-Saxons drank beer made from water and honeycomb, with herbs for flavoring.";
            bee_facts[44] = "The word “honeymoon” is derived from the ancient tradition of supplying a newlywed couple with a month’s supply of mead in order to ensure happiness and fertility.";
            bee_facts[45] =
                "Humans sometimes use the Greater Honeyguide to find bee hives in the wild.";
            bee_facts[46] =
                "While a worker bee will die after it stings, a queen can survive stinging.";
            bee_facts[47] = "Worker bees have barbed stingers, while a queen has a smooth stinger, which she mostly uses to kill other queens.";
            bee_facts[48] = "In the Hittite Empire (modern-day Turkey and Syria), a swarm’s value was equal to that of a sheep, and the penalty for bee thieving was a fine of 6 shekels of silver.";
            bee_facts[49] =
                "The Magna Carta legalized the harvesting of wild honey by common folk.";
            bee_facts[50] = "A hive will collect approximately 66 pounds of pollen per year.";
            bee_facts[51] = "A worker bee can carry a load of nectar or pollen equal to 80% of her own body weight.";
            bee_facts[52] = "Up until the mid-1700’s in England, it was common practice to kill all of the bees in a hive during honey collection.";
            bee_facts[53] =
                "For every pound of honey produced, a hive must collect 10 pounds of pollen.";
            bee_facts[54] = "In the United States, more than 300 different kinds of honey are produced every year. The variety in color and flavor is determined by the types of flowers from which the bees collect nectar.";
            bee_facts[55] = "The European honey bee was brought over to North America by the Shakers. Because of this, Native Americans referred to honey bees as the “White Man’s Fly”.";
            bee_facts[56] = "Honey bees did not spread to Alaska until 1927.";
            bee_facts[57] = "During the American Revolution, George Washington said “It was the cackling geese that saved Rome, but it was the bees that saved America.” Read the full story here.";
            bee_facts[58] = "Honey bees have 170 odorant receptors, and have a sense of smell 50 times more powerful than a dog.";
            bee_facts[59] = "Every bee colony has its own distinct scent so that members can identify each other.";
            bee_facts[60] = "A hive is perennial, meaning that it becomes inactive in the winter but “awakens” again in the spring. When individuals die, they are quickly replace – workers every 6-8 weeks, and the queen every 2-3 years. Because of this, a hive could technically be immortal!";
            bee_facts[61] = "Bees have 2 stomachs – one for eating, and one for storing nectar.Honey Bee Anatomy Diagram Bee Fact";
            bee_facts[62] = "Bees have existed for around 30 million years.";
            bee_facts[63] = "Hives produce 5 distinct substances: honey, beeswax, propolis, pollen, and royal jelly.";
            bee_facts[64] =
                "Newborn bees ask for food by sticking out their tongues at passing worker bees.";
            bee_facts[65] = "While bears do enjoy honey, they prefer to eat bee larvae.";
            bee_facts[66] = "Bees have long, straw-like tongues called a proboscis which they use to suck liquid nectar out of flowers.";
            bee_facts[67] = "During the winter, some worker bees take on the job of “heater bees,” where they vibrate their bodies in order to keep the hive at the optimal temperature of 95ºF.";
            bee_facts[68] = "Bees make honey by regurgitating digested nectar into honeycomb cells and then fanning it with their wings.";
            bee_facts[69] = "Honeycomb cells have many uses other than storing honey. They are also used to store nectar, pollen, and water, as well as a nursery for larvae!";
            bee_facts[70] = "Bees have 5 eyes – 3 simple eyes, and 2 compound eyes.";
            bee_facts[71] = "Only female bees have stingers.";
            bee_facts[72] = "The females do all of the work in the hive. The drones’ only job is to mate with a queen.";
            bee_facts[73] = "Bees have 4 life stages: egg, larvae, pupae, and adult.";
            bee_facts[74] =
                "Honey has antibacterial properties and can be used as a dressing for wounds.";
            bee_facts[75] = "The top-bar hive originated in Africa.";
            bee_facts[76] = "The queen bee (and only the queen) eats royal jelly for the duration of her life. This milky substance is produced in a special gland located in a worker bee’s head.";
            bee_facts[77] = "Bees use propolis, a sticky substance gathered from the buds of trees, to fill in cracks and weatherproof their hives.";
            bee_facts[78] = "Bees create wax in a special gland on their stomach, which they then chew to form honeycomb.";
            bee_facts[79] =
                "Bees communicate in 2 ways: the waggle dance, and through the use of pheromones.";
            bee_facts[80] = "Due to the rise in popularity of urban beekeeping, it is estimated that honey bees outnumber the residents of London 30-1 in the summer months.";
            bee_facts[81] = "Ever wonder why a beekeeper’s suit is always white? It’s because bees react strongly to dark colors!";
            bee_facts[82] = "The science of beekeeping is called “apiculture”.";
            bee_facts[83] =
                "In 1984, honeybees on a space shuttle constructed a honeycomb in zero gravity.";
            bee_facts[84] = "Primitive hives were made from earthenware, mud, or hollow logs.";
            bee_facts[85] = "Bees are sold by the pound.";
            bee_facts[86] = "Swarming occurs when a colony has outgrown its current hive and is preparing to separate into 2 or more new, smaller hives.";
            bee_facts[87] = "The top honey producing states are North Dakota, California, and South Dakota as of 2016.";
            bee_facts[88] = "A single ounce of honey could fuel a honey bee’s flight all the way around the world.";
            bee_facts[89] = "Honey bees are the only type of bee that dies after stinging.";
            bee_facts[90] = "Honey bees usually travel about 3 miles away from the hive in search of nectar and pollen.";
            bee_facts[91] = "Honey is composed of 80% sugars and 20% water.";
            bee_facts[92] = "During the winter, worker bees will take short “cleansing flights” in order to defecate and remove debris from the hive.";
            bee_facts[93] = "Some worker bees have the job of being an “undertaker bee” and are in charge of removing dead bees from the hive.";
            bee_facts[94] = "Due to colony collapse disorder, bees have been dying off at a rate of approximately 30% per year.";
            bee_facts[95] = "In Wisconsin, beekeepers can apply to have their honey certified as pure and use “Wisconsin certified honey” on their packaging.";
            bee_facts[96] = "Bees hate human breath.";
            bee_facts[97] = "Bees are being used to study dementia. When a bee takes on a new job usually done by a younger bee, its brain stops aging!";
            bee_facts[98] =
                "Bees have their own “facial recognition software,” and can recognize human faces.";
            bee_facts[99] = "Bee Laura Maigatter Facial Recognition";

            let mut votes = VoteObj::new(0, String::from(""));
            loop {
                //let locked_commands = cloned_commands.commands;
                let cmd = cr.recv().unwrap();
                //TODO insert proper logic for handling more than just !hello
                if command_reg.is_match(&cmd.text.trim()) {
                    let command = command_reg.find(&cmd.text.trim()).unwrap().as_str();
                    //TODO: svote, evote, extend, remember, vote, add command
                    match &command[1..] {
                        //special commands are placed in their own patterns in the match, while "regular" commands all go into default.
                        "hello" => rs
                            .send((format!("Hello, {}", cmd.user), cmd.channel))
                            .unwrap(),
                        "svote" => {}
                        "evote" => {}
                        "extend" => {}
                        "remember" => {}
                        "mytime" => {}
                        "add" => {}
                        "vote" => {
                            votes.has_started = true; //TODO: remove, only here for testing purposes
                            if votes.has_started {
                                if regex_collection.time_regex.is_match(&cmd.text) {
                                    let time = regex_collection
                                        .time_regex
                                        .find(&cmd.text)
                                        .unwrap()
                                        .as_str();
                                    votes.add_vote(
                                        String::from(cmd.user.as_str()),
                                        convert_string_int(String::from(time)),
                                    ); //TODO: get time through regex
                                    rs.send((
                                        format!("{} voted on the time {}", cmd.user, time),
                                        cmd.channel,
                                    ))
                                    .unwrap();
                                } else if regex_collection.number_regex.is_match(&cmd.text) {
                                    let number = regex_collection
                                        .number_regex
                                        .find(&cmd.text)
                                        .unwrap()
                                        .as_str();
                                    votes.add_vote(
                                        String::from(cmd.user.as_str()),
                                        convert_string_int(String::from(number)),
                                    ); //TODO: get time through regex
                                    rs.send((
                                        format!("{} voted on {}", cmd.user, number),
                                        cmd.channel,
                                    ))
                                    .unwrap();
                                } else {
                                    rs.send((
                                        format!(
                                            "@{} {}",
                                            cmd.user,
                                            commands.commands.get_mut(command).unwrap().help
                                        ),
                                        cmd.channel,
                                    ))
                                    .unwrap();
                                }
                            } else {
                                rs.send((
                                    format!(
                                        "@{} there is currently no active voting session",
                                        cmd.user
                                    ),
                                    cmd.channel,
                                ))
                                .unwrap()
                            }
                        }
                        "help" => {
                            //TODO: exchange with regex to find if it included a command to ask for
                            //help
                            if !regex_collection.help_regex.is_match(&cmd.text) {
                                let mut viewer_commands = String::from("Normal commands: ");
                                let mut moderator_commands =
                                    String::from("Moderator only commands: ");
                                let mut broadcaster_commands =
                                    String::from("Broadcaster only commands: ");

                                for (k, c) in &commands.commands {
                                    match c.elevation {
                                        Elevation::Viewer => {
                                            viewer_commands.push_str(&format!("{}, ", &k[1..]))
                                        }
                                        Elevation::Moderator => {
                                            moderator_commands.push_str(&format!("{}, ", &k[1..]))
                                        }
                                        Elevation::Broadcaster => {
                                            broadcaster_commands.push_str(&format!("{}, ", &k[1..]))
                                        }
                                    }
                                }
                                rs.send((
                                    format!(
                                        "{} {} {} ",
                                        viewer_commands, moderator_commands, broadcaster_commands
                                    ),
                                    cmd.channel,
                                ))
                                .unwrap();
                            } else {
                                let help_command = format!(
                                    "{}{}",
                                    &cloned_arabot_symbol,
                                    regex_collection
                                        .help_regex
                                        .captures(&cmd.text)
                                        .unwrap()
                                        .get(1)
                                        .unwrap()
                                        .as_str()
                                );
                                if command_reg.is_match(&help_command) {
                                    rs.send((
                                        format!(
                                            "{}",
                                            commands
                                                .commands
                                                .get_mut(help_command.as_str())
                                                .unwrap()
                                                .help
                                        ),
                                        cmd.channel,
                                    ))
                                    .unwrap();
                                }
                            }
                        }
                        "bee" => {
                            rs.send((
                                format!("{}", bee_facts.choose(&mut rand::thread_rng()).unwrap()),
                                cmd.channel,
                            ))
                            .unwrap();
                        }
                        "slots" => {
                            let emote1 = emote_list.choose(&mut rand::thread_rng()).unwrap();
                            let emote2 = emote_list.choose(&mut rand::thread_rng()).unwrap();
                            let emote3 = emote_list.choose(&mut rand::thread_rng()).unwrap();

                            if emote1.as_str() == emote2.as_str()
                                && emote2.as_str() == emote3.as_str()
                            {
                                rs.send((
                                    format!(
                                        "{} {} {} JACKPOT @{}",
                                        emote1, emote2, emote3, cmd.user
                                    ),
                                    cmd.channel,
                                ))
                                .unwrap();
                            } else {
                                rs.send((
                                    format!("{} {} {} @{}", emote1, emote2, emote3, cmd.user),
                                    cmd.channel,
                                ))
                                .unwrap();
                            }
                        }
                        _default => rs
                            .send((
                                format!(
                                    "{}",
                                    (commands.commands.get_mut(command).unwrap().response)(
                                        String::from(&cmd.user),
                                        cmd.text
                                    )
                                ),
                                String::from(&cmd.channel),
                            ))
                            .unwrap(), //rs.send((format!("Error occured: No command found"), cmd.channel)).unwrap(),
                    }
                }
                //if cmd.text.contains("!hello"){
                //
                //}
            }
        });

        let mut stream = client.stream()?;
        client
            .send(Command::CAP(
                None,
                CapSubCommand::REQ,
                Some(String::from("twitch.tv/tags")),
                None,
            ))
            .unwrap();

        let client = Arc::new(client);
        let cloned_client = Arc::clone(&client);
        let tmp_wait = self.message_wait;
        let answer_thread = thread::spawn(move || loop {
            let (response, channel) = rr.recv().unwrap();
            cloned_client.send_privmsg(&channel, &response).unwrap();
            thread::sleep(time::Duration::from_millis(tmp_wait));
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
    fn generate_regex(commands: &HashMap<String, ChatCommand>, command_symbol: &String) -> Regex {
        let mut command_reg: String = String::from(format!(r"^"));
        for i in commands.keys() {
            //        command_reg.push(format!(r"{}{}", command_symbol, commands[i].command))
            command_reg
                .push_str(format!(r"^{}{}\b|", command_symbol, commands[i].command).as_str());
        }
        command_reg.push_str(format!(r"^{}hello\b", command_symbol).as_str());
        command_reg.push_str(r"{1}");
        regex::Regex::new(&command_reg).unwrap()
    }
}
fn convert_int_string(time: i64) -> String {
    String::from("")
}
fn convert_string_int(time: String) -> i64 {
    0
}
