// File: main.rs
// TODO: split commands into different files

extern crate dotenv;

use dotenv::dotenv;

use std::{
    collections::HashSet, env
};

use serenity::{
    model::{channel::Message, gateway::Ready},
    prelude::*,
    // utils::MessageBuilder,
    framework::standard::{
        StandardFramework, Args, CommandOptions,
        CheckResult, CommandResult,
        macros::{command, group, help, check},
    },
};

#[group]
#[commands(ping)]
struct General;

#[group]
#[prefixes("test")]
#[description = "Message Test"]
#[default_command(random_message)]
#[commands(dm, mention_me)]
struct Test;

#[group]
#[owners_only]
// Limit command to be guild-restricted.
#[only_in(guilds)]
#[commands(shutdown)]
struct Owner;

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _: Context, rdy: Ready) {
        println!("{} is ready", rdy.user.name);
    }
}

fn main() {
    // Configure the client with Discord bot token in the environment.
    dotenv().ok();
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment.");
    let mut client = Client::new(&token, Handler).expect("Error on creating client");

    // fetch bot's owners and id
    let (owners, bot_id) = match client.cache_and_http.http.get_current_application_info() {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    // Commands with common prefix
    client.with_framework(StandardFramework::new()
                          // set the bot's prefix to "!"
                          .configure(|c| c
                                     .with_whitespace(true)
                                     .on_mention(Some(bot_id))
                                     .prefix("!")
                                     .owners(owners))
                          // Command execution
                          .before(|ctx, msg, command_name| {
                              println!("Got command '{}' by user '{}'", command_name, msg.author.name);
                              true
                          })
                          .after(|_, _, command_name, error| {
                              match error {
                                  Ok(()) => println!("Processed command '{}'", command_name),
                                  Err(why) => println!("Command '{}' returned error: {:?}", command_name, why),
                              }
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
                          .group(&GENERAL_GROUP)
                          .group(&TEST_GROUP)
                          .group(&OWNER_GROUP));

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}

#[command]
fn ping(ctx: &mut Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!");

    Ok(())
}

#[command]
fn random_message(ctx: &mut Context, msg: &Message) -> CommandResult {
    // TODO: send a random/static message to channel without mention anyone

    Ok(())
}

#[command]
fn dm(ctx: &mut Context, msg: &Message) -> CommandResult {
    let dm = msg.author.dm(&ctx, |m| {
        m.content("安安");

        m
    });

    if let Err(why) = dm {
        println!("Error when direct messaging user: {:?}", why);
    }

    Ok(())
}

#[command]
fn mention_me(ctx: &mut Context, msg: &Message) -> CommandResult {
    // TODO: mention the original author

    Ok(())
}

#[command]
fn shutdown(ctx: &mut Context, msg: &Message) -> CommandResult {
    // TODO: find a way to shutdown this bot with command

    let _ = msg.reply(&ctx, "Shutting down!");

    Ok(())
}

#[check]
#[name = "Owner"]
fn owner_check(_: &mut Context, msg: &Message, _: &mut Args, _: &CommandOptions) -> CheckResult {
    (msg.author.id == 273036229531009024).into()
}

#[check]
#[name = "Admin"]
fn admin_check(ctx: &mut Context, msg: &Message, _: &mut Args, _: &CommandOptions) -> CheckResult {
    if let Some(member) = msg.member(&ctx.cache) {
        if let Ok(permissions)= member.permissions(&ctx.cache) {
            return permissions.administrator().into();
        }
    }

    false.into()
}
