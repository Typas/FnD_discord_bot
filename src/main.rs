// File: main.rs

use dotenv::dotenv;

use std::{collections::HashSet, env};

use serenity::{
    client::{Client, Context, EventHandler},
    framework::standard::StandardFramework,
    model::gateway::Ready,
};

use fnd::{admin::*, member::*};

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _: Context, rdy: Ready) {
        println!("{} is ready", rdy.user.name);
    }
}

fn main() {
    // Configure the client with Discord bot token in the environment.
    dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment.");
    let mut client = Client::new(&token, Handler).expect("Error on creating client");

    // fetch bot's owners and id
    let (owners, bot_id) = match client.cache_and_http.http.get_current_application_info() {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    // Commands with common prefix
    client.with_framework(
        StandardFramework::new()
            // set the bot's prefix to "!"
            .configure(|c| {
                c.with_whitespace(true)
                    .on_mention(Some(bot_id))
                    .prefix("!")
                    .owners(owners)
            })
            // Command execution
            .before(|_ctx, msg, command_name| {
                println!(
                    "Got command '{}' by user '{}'",
                    command_name, msg.author.name
                );
                true
            })
            .after(|_, _, command_name, error| match error {
                Ok(()) => println!("Processed command '{}'", command_name),
                Err(why) => println!("Command '{}' returned error: {:?}", command_name, why),
            })
            // Command exception
            .unrecognised_command(|_, _, unknown_command_name| {
                println!("Could not find command named '{}'", unknown_command_name);
            })
            // .normal_message(|_, message| {
            //     println!("Message '{}' is not a command", message.content);
            // })
            // TODO: help message
            // .help(&NOTHING_HELP)
            // set groups
            .group(&MEMBER_GROUP)
            .group(&ADMIN_GROUP),
    );

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}
