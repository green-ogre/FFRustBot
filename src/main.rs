
use rand::Rng;
use std::env;
use std::time::Duration;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::user;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;
use serenity::model::prelude::ChannelId;


struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    async fn message(&self, context: Context, msg: Message) {
        if msg.content == "!challange" {

            // let channel = match channel_info.channel_id.to_channel(&channel_info.context).await {
            //     Ok(channel) => channel,
            //     Err(why) => {
            //         println!("Error getting channel: {:?}", why);
            //         return;
            //     },
            // };

            let response = MessageBuilder::new()
                .push_italic("Generating game...")
                .build();

            if let Err(why) = msg.channel_id.say(&context.http, response).await {
                println!("Error sending message: {:?}", why);
            }

            let channel_info = ChannelWrite{
                channel_id: msg.channel_id,
                context,
                player_name: msg.author.name.clone(),
                user: msg.author
            };

            init_game(&channel_info).await;

        } 
        else if msg.content == "!test" {
            let action = Action::from(10);
            let response = format!("{:?}", &action);
            if let Err(why) = msg.channel_id.say(&context.http, response).await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}


#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to load .env file");
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot. This will
    // automatically prepend your bot token with "Bot ", which is a requirement
    // by Discord for bot users.
    let mut client =
        Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}



struct ChannelWrite {
    channel_id: ChannelId,
    context: Context,
    player_name: String,
    user: user::User
}


struct Stats {
    max_health: i32,
    health: i32,
    block: i32
}

struct Entity {
    name: String,
    ascii: String,
    stats: Stats,
}

impl Entity {
    fn recieve_attack(&mut self, dmg: i32) {
        let mut apply_dmg = dmg;
        if self.stats.block > 0 {
            apply_dmg -= self.stats.block;
            self.stats.block = 0;
        }
        if apply_dmg < 0 {
            apply_dmg = 0;
        }
        self.stats.health -= apply_dmg;
    }

    fn block_next_action(&mut self, _block: i32) {
        self.stats.block = _block;
    }

    fn heal(&mut self, _heal: i32) {
        self.stats.health += _heal;
        if self.stats.health > self.stats.max_health {
            self.stats.health = self.stats.max_health;
        }
    }
}

#[derive(Debug)]
enum Action {
    Attack,
    Block,
    Heal,
    None
}


impl Action {
    fn from(action: u8) -> Action {
        match action {
            1 => Action::Attack,
            2 => Action::Block,
            3 => Action::Heal,
            _ => Action::None
        }
    }

    async fn perform_action(&self, this: &mut Entity, other: &mut Entity, channel_info: &ChannelWrite) -> &str {
        // Debug names
        let this_name = &this.name.trim();
        let other_name = &other.name.trim();

        // Roll is between 1-6
        let roll = roll();
        match self {
            Action::Attack => {

                let response = MessageBuilder::new()
                    .push(this_name)
                    .push(" hit ")
                    .push(other_name)
                    .push(" for ")
                    .push(roll)
                    .push(" damage...")
                    .build();
                let _ = channel_info.channel_id.say(&channel_info.context.http, &response).await.unwrap();

                other.recieve_attack(roll);
                "(ง ͠° ͟ل͜ ͡°)ง"
            },
            Action::Block => {

                let response = MessageBuilder::new()
                    .push(this_name)
                    .push(" is blocking from ")
                    .push(roll)
                    .push(" damage...")
                    .build();
                let _ = channel_info.channel_id.say(&channel_info.context.http, &response).await.unwrap();

                this.block_next_action(roll);
                "ᕙ(⇀ ‸ ↼‶)ᕗ"
            },
            Action::Heal => {
                
                let response = MessageBuilder::new()
                    .push(this_name)
                    .push(" is healing for ")
                    .push(roll)
                    .push(" health...")
                    .build();
                let _ = channel_info.channel_id.say(&channel_info.context.http, &response).await.unwrap();

                this.heal(roll);
                "(๑❛ ڡ ❛๑)  "
            },
            Action::None => { 
                
                let response = MessageBuilder::new()
                    .push(this_name)
                    .push(" does nothing... For some reason?")
                    .build();
                let _ = channel_info.channel_id.say(&channel_info.context.http, &response).await.unwrap();

                "ヽ(゜～゜o)ノ"
            }
        }
    }
}


fn roll() -> i32 {
    rand::thread_rng().gen_range(1..7)
}


async fn init_game(channel_info: &ChannelWrite) {
    // let player_names = ["Adventurer:", "Knight:    ", "Noble:     ", "Mage:      ", "Spy:       ", "Ranger:    "];
    let enemy_names = ["Goblin:", "Elf:   ", "Troll: ", "Ghoul: ", "Dwarf: ", "Ork:   "];

    // Create the player and enemy entities
    let player_stats = Stats {
        max_health: 20,
        health: 20,
        block: 0
    };

    // Since these are primitive types, they will be copied
    // instead of referenced
    let enemy_stats = Stats { ..player_stats };

    let player = Entity {
        name: channel_info.player_name.clone(),
        ascii: String::from("ヽ(゜～゜o)ノ"),
        stats: player_stats,
    };

    let enemy = Entity {
        name: String::from(enemy_names[roll() as usize - 1]),
        ascii: String::from("ヽ(゜～゜o)ノ"),
        stats: enemy_stats,
    };

    game_loop(player, enemy, channel_info).await;
}


async fn read_input(channel_info: &ChannelWrite) -> Result<u8, std::num::ParseIntError> {
    let input: String;
    if let Some(answer) = &channel_info.user.await_reply(&channel_info.context).timeout(Duration::from_secs(60)).await {
        if answer.content.to_lowercase() == "!1" {
            input = String::from("1");
            println!("!1");
        } 
        else if answer.content.to_lowercase() == "!2" {
            input = String::from("2");
            println!("!2");
        }
        else if answer.content.to_lowercase() == "!3" {
                input = String::from("3");
                println!("!3");
        } else {
            input = String::from("None");
            println!("not recognized");
        }
    } else {
        // No input provided in 60 seconds
        input = String::from("None");
        let _ = channel_info.channel_id.say(&channel_info.context.http, "No answer provided in 60 seconds, did you forget about me?").await;
    };

    // If parse returns an error, then read_input will be called again in the game_loop
    let result = input.parse::<u8>()?;
    Ok(result)
}


async fn commit_player_action(player_action: u8, player: &mut Entity, enemy: &mut Entity, channel_info: &ChannelWrite) {
    // Match the desired input with an action
    let action = Action::from(player_action);

    // Perform the action and update the player ascii
    let player_ascii = action.perform_action(player, enemy, channel_info).await;
    player.ascii = String::from(player_ascii);
}


async fn commit_enemy_action(enemy_action: i32, player: &mut Entity, enemy: &mut Entity, channel_info: &ChannelWrite) {
    // Match the desired input with an action
    let action = Action::from(enemy_action as u8);    // Note that the enemy will never commit action 'None'

    // Perform the action and update the enemy ascii
    let enemy_ascii = action.perform_action(enemy, player, channel_info).await;
    enemy.ascii = String::from(enemy_ascii);
}


async fn game_loop(mut player: Entity, mut enemy: Entity, channel_info: &ChannelWrite) {
    // Initial print
    print_arena(&player, &enemy, &channel_info).await;

    // Game loop
    'game_loop: loop {
        // Parse user input until valid
        loop {
            let mut game_over = false;

            // Transforms the input into an integer and handles any errors
            let player_input = read_input(channel_info).await;
            let player_action = match player_input {
                Ok(v) => v,
                Err(_e) => {
                    let _ = channel_info.channel_id.say(&channel_info.context.http, "That's not an option is it? Why must you make me handle this error!").await;
                    break;
                }
            };

            // Generates enemies action as an integer
            let enemy_action = rand::thread_rng().gen_range(1..4);

            // Changes the order of actions committed based on who is blocking
            if enemy_action == 2 && player_action == 1 {
                commit_enemy_action(enemy_action, &mut player, &mut enemy, channel_info).await;
                commit_player_action(player_action, &mut player, & mut enemy, channel_info).await;
            } else {
                commit_player_action(player_action, &mut player, &mut enemy, channel_info).await;
                commit_enemy_action(enemy_action, &mut player, &mut enemy, channel_info).await;
            }

            // Check if either players should die
            if player.stats.health <= 0 {
                player.ascii = String::from("(︶︹︺)    ");
                player.stats.health = 0;
                game_over = true;
            }
            if enemy.stats.health <= 0 {
                enemy.ascii = String::from("(︶︹︺)    ");
                enemy.stats.health = 0;
                game_over = true;
            }

            // Print updated arena
            print_arena(&player, &enemy, &channel_info).await;

            if game_over {
                let _ = channel_info.channel_id.say(&channel_info.context.http, "Game Over...").await;
                break 'game_loop;
            }
        }
    }
}


async fn print_arena(player: &Entity, enemy: &Entity, channel_info: &ChannelWrite) {
    let response = MessageBuilder::new()
        .push_line(" ")
        .push_line(" ")
        .push_bold(&player.name)
        .push("\t\t\t\t\t\t  ")
        .push_bold(&enemy.name)
        .push("\n\n")
        .push(&player.ascii)
        .push("\t\t\t\t\t\t  ")
        .push(&enemy.ascii)
        .push("\n[")
        .push_italic(player.stats.health)
        .push("/")
        .push_italic(player.stats.max_health)
        .push("]\t\t\t\t\t\t\t  [")
        .push_italic(enemy.stats.health)
        .push("/")
        .push_italic(enemy.stats.max_health)
        .push("]\n\nActions:\n1 -- Attack\t\t2 -- Block\n3 -- Heal")
        .push_line(" ")
        .push_line(" ")
        .build();
    if let Err(why) = channel_info.channel_id.say(&channel_info.context.http, response).await {
        println!("Error sending message: {:?}", why);
    }
}