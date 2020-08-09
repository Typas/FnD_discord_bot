use crate::util;
use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandError, CommandResult,
    },
    model::channel::Message,
    model::id::RoleId,
    prelude::*,
};
use std::collections::HashMap;

#[group]
#[commands(convert_role)]
pub struct Member;

#[command]
#[aliases("mention")]
// FIXME: what the hell is these code
// TODO: accept role name as arg
pub fn convert_role(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let raw_str = args.raw().collect::<Vec<&str>>();
    // FIXME: arg_to_roleid() only takes id
    // TODO: replace with more general method, which can accept role name
    let roles = match raw_str.len() {
        0 => Vec::new(),
        _ => raw_str
            // .into_iter()
            .iter()
            .map(|x| util::arg_to_roleid(x, ctx, msg).unwrap_or(RoleId::from(0)))
            .filter(|x| x != &RoleId::from(0))
            .collect(),
    };
    let _t: HashMap<_, _> = msg
        .guild(&ctx.cache)
        .ok_or(CommandError(String::from("hell")))?
        .read()
        .roles
        .iter()
        .map(|(a, b)| (&b.name, a))
        .collect();
    match raw_str.len() {
        0 => (),
        _ => match util::arg_to_roleid(raw_str[0], ctx, msg) {
            Ok(_) => (),
            Err(e) => eprintln!("msg: {:?}, content: {}", e, raw_str[0]),
        },
    };
    let gid = match msg.guild_id {
        Some(n) => n,
        None => return Err(CommandError(String::from("Missing guild id"))),
    };
    let rp = util::get_role_pairs(&gid)?;
    let rp = match roles.len() {
        0 => rp,
        _ => rp
            .into_iter()
            .filter(|(k, v)| roles.contains(k) || roles.contains(v))
            .collect(),
    };
    let rp_rev: HashMap<_, _> = rp.clone().into_iter().map(|(a, b)| (b, a)).collect();

    // if any muted role -> convert all to mentionable
    // else convert all to immentionable
    let mut any = false;
    let mut user = msg
        .member(&ctx.cache)
        .ok_or(CommandError(String::from("role not found")))?;
    let mut to_rm = Vec::new();
    let mut to_add = Vec::new();
    for r in &user.roles {
        if let Some(v) = rp_rev.get(r) {
            any = true;
            to_add.push(*v);
            to_rm.push(*r);
        }
    }
    if any == false {
        for r in &user.roles {
            if let Some(v) = rp.get(r) {
                to_add.push(*v);
                to_rm.push(*r);
            }
        }
    }
    user.add_roles(&ctx.http, &to_add)?;
    user.remove_roles(&ctx.http, &to_rm)?;

    let reply_message = format!(
        "Roles have been changed to {}",
        if any {
            "mentionable"
        } else {
            "not mentionable"
        }
    );

    if let Err(why) = msg.channel_id.say(&ctx.http, &reply_message) {
        eprintln!("Error sending message: {:?}", why);
    }

    Ok(())
}
