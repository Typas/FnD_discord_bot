use serenity::{
    client::Context,
    framework::standard::{
        macros::{check, command, group},
        Args, CheckResult, CommandOptions, CommandResult, CommandError,
    },
    model::channel::Message,
};
use crate::util;

#[group]
#[only_in(guilds)]
#[checks(Admin)]
#[commands(add_role_pair, random_auth_code)]
pub struct Admin;

#[check]
#[name = "Admin"]
fn admin_check(ctx: &mut Context, msg: &Message, _: &mut Args, _: &CommandOptions) -> CheckResult {
    if let Some(member) = msg.member(&ctx.cache) {
        if let Ok(permissions) = member.permissions(&ctx.cache) {
            return permissions.administrator().into();
        }
    }

    false.into()
}

#[command]
#[aliases("pair")]
pub fn add_role_pair(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let raw_str = args.raw().collect::<Vec<&str>>();
    let rolepair = match raw_str.len() {
        0..=1 => return Err(CommandError(String::from("Missing arguments"))),
        _ => (util::arg_to_roleid(raw_str[0], ctx, msg), util::arg_to_roleid(raw_str[1], ctx, msg)),
    };
    let gid = match msg.guild_id {
        Some(n) => n,
        None => return Err(CommandError(String::from("Missing guild id"))),
    };
    let rolepair = match rolepair {
        (Ok(a), Ok(b)) => (a, b),
        (Err(e), _) => {
            msg.channel_id.say(&ctx.http, format!("Cannot recognize role \"{}\"", raw_str[0]))?;
            return Err(e);
        },
        (_, Err(e)) => {
            msg.channel_id.say(&ctx.http, format!("Cannot recognize role \"{}\"", raw_str[1]))?;
            return Err(e);
        },
    };

    match util::add_one_role_pair(&gid, rolepair) {
        Ok(_) => {
            msg.channel_id.say(&ctx.http, format!("Pair ({}, {}) added", rolepair.0, rolepair.1))?;
            Ok(())},
        Err(e) => Err(e),
    }
}

// 目前固定為六碼長度，會自動生成需要的訊息。
// TODO: 在一小時內可以自動從 #認證區 取得對應驗證碼並給予基礎身分組。
#[command]
#[aliases("auth")]
pub fn random_auth_code(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    const LEN_CODE: usize = 6;
    let code = util::string_generator(LEN_CODE);
    let raw_str: Vec<&str> = args.raw().collect();
    let auth_account_name = match raw_str.len() {
        0 => "神祕人士",
        _ => raw_str[0],
    };

    let reply_message = format!("寄給：\n\n\
                                 {}\n\n\
                                 標題：\n\n\
                                 副本找團區認證信\n\n\
                                 內文：\n\n\
                                 歡迎來到副本找團區，驗證碼如下共6碼\n\
                                 {}\n\
                                 直接貼到認證區就行了
                                 ",
                                auth_account_name,
                                code);

    if let Err(why) = msg.channel_id.say(&ctx.http, &reply_message) {
        eprintln!("Error sending message: {:?}", why);
    }

    Ok(())
}
