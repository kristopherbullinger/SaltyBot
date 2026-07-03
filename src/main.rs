use chrono::{offset::Utc, DateTime, Datelike, Duration, Weekday};
use serenity::{
    async_trait,
    model::{
        channel::{Message, Reaction, ReactionType},
        gateway::{GatewayIntents, Ready},
        timestamp::Timestamp,
    },
    prelude::*,
};
use std::borrow::Cow;
use std::convert::TryFrom;

mod command;
mod glossary;
mod utils;
use command::Command;

const KINGCORD_GUILD_ID: u64 = 350242625502052352;
const SELF_USER_ID: u64 = 751611106107064451;
const SPEEZ_USER_ID: u64 = 442321800416854037;
const MOD_ROLE_ID: u64 = 432017127810269204;
const ADMIN_ROLE_ID: u64 = 350362647989846026;
const KINGCORD_INSTABAN_CHANNEL_ID: u64 = 1428557360518926418;
static CONSUL_ROLE_IDS: &'static [u64] = &[
    ADMIN_ROLE_ID,
    MOD_ROLE_ID,
    885971978052325376, //council
];
static NECO_ARC_DOUGIE: &str = "https://cdn.discordapp.com/attachments/350242625502052353/1010292204201332778/EynKWlUtroS3hAf4.mp4";
static NECO_ARC_SMOKING: &str = "https://pbs.twimg.com/media/FE6QLYLXEAg-ccT.jpg";
static NECO_ARC_BLOODY_AXE: &str =
    "https://i.pinimg.com/564x/22/40/08/224008b647e5f9ac8a158df7063b130d.jpg";
static NECO_ARC_SEATBELT: &str = "https://cdn.discordapp.com/attachments/350242625502052353/1090717976765931530/20230329_104015.png";

struct Handler;
const ONE_DAY: i64 = 24 * 60 * 60;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        // ban user if they post in the bot trap channel
        if msg.channel_id.get() == KINGCORD_INSTABAN_CHANNEL_ID
            && msg.author.id.get() != SELF_USER_ID
        {
            let member = match msg.member(&ctx.http).await {
                Ok(mem) => mem,
                Err(_) => {
                    return;
                }
            };
            let id = member.user.id.get();
            match member
                .ban_with_reason(&ctx.http, 1, "posted in instaban channel")
                .await
            {
                Ok(_) => {
                    let name = member.user.name;
                    let _ = msg
                        .channel_id
                        .say(&ctx.http, format!("ELIMINATED SCUM {}", name))
                        .await;
                }
                Err(e) => {
                    log::error!("failed to ban user <@{}>: {:?}", id, e);
                    return;
                }
            }
        }
        let content = msg.content.as_str();
        //if message is profane and sent in kingcord, silence user
        if msg.guild_id.map(|g| g.get()) == Some(KINGCORD_GUILD_ID) && utils::is_profane(content) {
            let mut member = match msg.member(&ctx.http).await {
                Ok(m) => m,
                Err(_) => return,
            };
            let until = Timestamp::from_unix_timestamp(Timestamp::now().unix_timestamp() + ONE_DAY)
                .unwrap();
            let _ = member
                .disable_communication_until_datetime(&ctx.http, until)
                .await;
        }
        let command = match Command::try_from(content) {
            Ok(c) => c,
            _ => return,
        };
        match command {
            Command::Friday => {
                let now: DateTime<Utc> = Utc::now();
                //approximately Texas
                let texas_utc_offset = Duration::hours(5);
                let texas_time = now - texas_utc_offset;
                let weekday = texas_time.weekday();
                let response = match weekday {
                    Weekday::Fri => {
                        if now.day() == 13 {
                            NECO_ARC_BLOODY_AXE
                        } else {
                            NECO_ARC_DOUGIE
                        }
                    }
                    Weekday::Sat if msg.author.id.get() == SPEEZ_USER_ID => NECO_ARC_SMOKING,
                    _ => NECO_ARC_SEATBELT,
                };
                let _ = msg.channel_id.say(&ctx.http, response).await;
            }
            Command::Glossary(term) => {
                let term = term.to_ascii_lowercase();
                let glossary_entry = glossary::get(term);
                let response = match glossary_entry {
                    Some(entry) => Cow::Owned(format!("```\n{}```", entry.def.as_str())),
                    None => Cow::Borrowed("```\nTerm Not Found\n```"),
                };
                let _ = msg.channel_id.say(&ctx.http, response.as_ref()).await;
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    async fn reaction_remove(&self, _ctx: Context, _reaction: Reaction) {}

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        let guild = match reaction.guild_id {
            Some(g) if g.get() == KINGCORD_GUILD_ID => g,
            _ => {
                return;
            }
        };
        let message = match reaction.message(&ctx.http).await {
            Ok(msg) => msg,
            Err(e) => {
                log::debug!("Failed to get message: {:?}", e);
                return;
            }
        };
        if !reaction.emoji.unicode_eq("👎") {
            return;
        }
        if !reaction
            .member
            .as_ref()
            .and_then(|mem| {
                mem.roles
                    .iter()
                    .copied()
                    .find(|role| CONSUL_ROLE_IDS.contains(&role.get()))
            })
            .is_some()
        {
            return;
        }
        let downvote_count = message
            .reactions
            .iter()
            .find(|reac| reac.reaction_type.unicode_eq("👎"))
            .map(|reac| reac.count)
            .unwrap_or_default();

        if downvote_count < 3 {
            return;
        }

        let mut member = match guild.member(&ctx.http, message.author.id).await {
            Ok(m) => m,
            Err(e) => {
                log::debug!("Failed to fetch message author: {:?}", e);
                return;
            }
        };

        let until =
            Timestamp::from_unix_timestamp(Timestamp::now().unix_timestamp() + ONE_DAY).unwrap();
        match member
            .disable_communication_until_datetime(&ctx.http, until)
            .await
        {
            Ok(_) => {
                //generate message to indicate timeout action
                let reacter_id = match reaction.user_id {
                    Some(s) => format!("{}", s),
                    None => "Someone".to_string(),
                };
                let author_id = message.author.id.get();
                let notif = format!(
                    "<@{}> has sent <@{}> to the Shadow Realm",
                    reacter_id, author_id
                );
                if let Err(e) = reaction.channel_id.say(&ctx.http, notif).await {
                    log::debug!("Failed to send message to channel: {:?}", e);
                    return;
                }
                //remove all thumbsdowns from message
                if let Err(e) = message
                    .delete_reaction_emoji(&ctx.http, ReactionType::Unicode("👎".into()))
                    .await
                {
                    log::debug!("Failed to delete thumbsdowns: {:?}", e);
                    return;
                }
            }
            Err(e) => {
                log::debug!("Failed to add timeout role to message author: {:?}", e);
                return;
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();
    glossary::init()?;

    const TOKEN: &str = include_str!("token.txt");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(TOKEN, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
    Ok(())
}
