use std::fs::File;
use std::collections::HashMap;
use std::io::prelude::*;
use byteorder::{ByteOrder, LittleEndian};
use rand::seq::SliceRandom;
use regex::Regex;
use serenity::{
    framework::standard::CommandError,
    model::id::{ChannelId, GuildId, MessageId, RoleId, UserId},
    model::channel::Message,
    client::Context,
};

pub type IdResult<T> = Result<T, CommandError>;

// pure u64
#[allow(dead_code)]
pub fn arg_to_messageid(arg: &str) -> IdResult<MessageId> {
    let n = arg.parse::<u64>();
    match n {
        Ok(n) => Ok(MessageId::from(n)),
        Err(e) => Err(CommandError(e.to_string())),
    }
}

// <#id>
#[allow(dead_code)]
pub fn arg_to_channelid(arg: &str) -> IdResult<ChannelId> {
    let r = Regex::new("<#[0-9]+>").unwrap();
    let n = regex_find_u64(arg, &r)?;
    let id = ChannelId::from(n);
    Ok(id)
}

// <@&id>
#[allow(dead_code)]
pub fn arg_to_roleid(arg: &str, ctx: &Context, msg: &Message) -> IdResult<RoleId> {
    let r = Regex::new("<@*&[0-9]+>").unwrap();
    let n = regex_find_u64(arg, &r);
    let id = match n {
        Ok(n) => RoleId::from(n),
        Err(_) => {
            // try converting name to id
            // FIXME: really dirty hack
            let r = msg.guild(&ctx.cache).ok_or(CommandError(String::from("hell")))?;
            let rr = r.read();
            let m: HashMap<&str, &RoleId> = rr.roles.iter().map(|(a, b)| (b.name.as_str(), a)).collect();
            match m.get(&arg) {
                Some(id) => **id,
                None => return Err(CommandError(format!("Cannot parse argument {}", arg))),
            }
        },
    };
    Ok(id)
}

// <@!id>
#[allow(dead_code)]
pub fn arg_to_userid(arg: &str, ctx: &Context, msg: &Message) -> IdResult<UserId> {
    let r = Regex::new("<@*![0-9]+>").unwrap();
    let n = regex_find_u64(arg, &r);
    let id = match n {
        Ok(n) => UserId::from(n),
        Err(_) => {
            // try converting name to id
            // FIXME: really dirty hack
            let r = msg.guild(&ctx.cache).ok_or(CommandError(String::from("hell")))?;
            let rr = r.read();
            let m: HashMap<String, &UserId> = rr.members.iter().map(|(a, b)| (b.display_name().to_string(), a)).collect();
            match m.get(arg) {
                Some(id) => **id,
                None => return Err(CommandError(format!("Cannot parse argument {}", arg))),
            }
        },
    };
    Ok(id)
}

#[allow(dead_code)]
pub fn regex_find_u64(arg: &str, re: &Regex) -> IdResult<u64> {
    let err = CommandError(format!("Cannot parse from argument {}", arg));
    let r = Regex::new("[0-9]+").unwrap();

    match arg.parse::<u64>() {
        Ok(n) => return Ok(n),
        Err(_) => (),
    }

    let tmp = re.find(arg);
    let s = match tmp {
        Some(_) => r.find(arg),
        None => return Err(err),
    };
    let n = match s {
        Some(s) => s.as_str().parse::<u64>(),
        None => return Err(err),
    };

    match n {
        Ok(n) => Ok(n),
        Err(_) => Err(err),
    }
}

pub fn string_generator(len: usize) -> String {
    static RNG_BASES: &[u8] = "abcdefghijkmnpqrstuvwxyzABDEFGHJMNQRTY1234567890".as_bytes();
    let mut rng = &mut rand::thread_rng();
    let result: String = RNG_BASES
        .choose_multiple(&mut rng, len)
        .copied()
        .map(|x| x as char)
        .collect::<String>();

    result
}

pub type RolePair = (RoleId, RoleId);
pub type RolePairResult = Result<HashMap<RoleId, RoleId>, CommandError>;

// FIXME: use database
fn role_pair_path(guild: &GuildId) -> String {
    let s = format!("config/.{}-role-pairs.hell", guild.as_u64());
    String::from(s)
}

fn role_pair_file(guild: &GuildId) -> std::io::Result<File> {
    let path = role_pair_path(guild);
    let f = File::open(path)?;

    Ok(f)
}

// FIXME: should use serde, not hard coded hell
pub fn get_role_pairs(guild: &GuildId) -> RolePairResult {
    let mut rp = Vec::new();
    let mut f = match role_pair_file(guild) {
        Ok(f) => f,
        Err(e) => return Err(CommandError(e.to_string())),
    };

    let mut tmp: Vec<u8> = Vec::new();
    match f.read_to_end(&mut tmp) {
        Ok(_) => (),
        Err(e) => return Err(CommandError(e.to_string())),
    }

    for i in (0..tmp.len()).step_by(16) {
        let f = |n: usize| -> RoleId {
            let id = LittleEndian::read_u64(&tmp[n..n+8]);
            RoleId::from(id)
        };
        let a = f(i);
        let b = f(i+8);
        rp.push((a, b));
    }

    let result = rp.into_iter().collect();

    Ok(result)
}

// FIXME: should use serde, not hard coded hell
pub fn add_one_role_pair(guild: &GuildId, new_pair: RolePair) -> Result<(), CommandError> {
    let mut rp = match get_role_pairs(guild) {
        Ok(p) => p,
        Err(_) => HashMap::new(),
    };

    rp.entry(new_pair.0).or_insert(new_pair.1);

    let fname = role_pair_path(guild);
    let mut f = match File::create(fname) {
        Ok(f) => f,
        Err(e) => return Err(CommandError(e.to_string())),
    };

    let rp_storage: Vec<RolePair> = rp.into_iter().collect();

    let mut write_in_bytes = |id: &RoleId| -> Result<(), CommandError> {
        match f.write(&id.as_u64().clone().to_le_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => Err(CommandError(e.to_string())),
        }
    };

    for p in &rp_storage {
        write_in_bytes(&p.0)?;
        write_in_bytes(&p.1)?;
    }

    Ok(())
}
