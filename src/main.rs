use std::borrow::Cow;
use std::convert::TryFrom;

use chrono::{offset::Utc, DateTime, Datelike, Duration, Weekday};
use rand::{thread_rng, Rng};
use serenity::{
    async_trait,
    model::{
        channel::{Message, Reaction, ReactionType},
        gateway::Ready,
    },
    prelude::*,
};

mod command;
mod glossary;
mod utils;
use command::{Command, FRIDAY_GIFS, QUOTES};

const KINGCORD_GUILD_ID: u64 = 350242625502052352;
const KINGCORD_TIMEOUT_ROLE_ID: u64 = 547814221325271072;
const SELF_USER_ID: u64 = 751611106107064451;
const SPEEZ_USER_ID: u64 = 442321800416854037;
static CONSUL_ROLE_IDS: &'static [u64] = &[
    350362647989846026, //admin
    432017127810269204, //moderator
    885971978052325376, //council
];
static RANDOM_FROG_URL: &str = "https://source.unsplash.com/450x400/?frog";

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let content = msg.content.as_str();
        //if message is profane and sent in kingcord, silence user
        if msg.guild_id.map(|g| g.0) == Some(KINGCORD_GUILD_ID) && utils::is_profane(content) {
            let author_id = msg.author.id.0;
            let http = ctx.http.clone();
            let _ = http
                .add_member_role(KINGCORD_GUILD_ID, author_id, KINGCORD_TIMEOUT_ROLE_ID)
                .await;
            return;
        }
        let command = match Command::try_from(content) {
            Ok(c) => c,
            _ => return,
        };
        match command {
            Command::Salt => {
                let response = {
                    let mut rng = thread_rng();
                    let quote = QUOTES[rng.gen_range(0..QUOTES.len())];
                    format!("```py\n'''\n{}\n'''```", quote)
                };
                let _ = msg.channel_id.say(&ctx.http, response).await;
            }
            Command::Friday => {
                let now: DateTime<Utc> = Utc::now();
                //approximately Texas
                let texas_utc_offset = Duration::hours(5);
                let texas_time = now - texas_utc_offset;
                let weekday = texas_time.weekday();
                let response = match weekday {
                    Weekday::Fri => {
                        let mut rng = thread_rng();
                        let quote = FRIDAY_GIFS[rng.gen_range(0..FRIDAY_GIFS.len())];
                        Cow::Owned(format!("it's motha fucken FRIDAY!!\n{}", quote))
                    }
                    Weekday::Mon if msg.author.id.0 == SPEEZ_USER_ID => Cow::Borrowed(
                        "https://pbs.twimg.com/media/FAoSsRdVEAQwJ9Y?format=png&name=900x900",
                    ),
                    _ => Cow::Borrowed("it is not friday"),
                };
                let _ = msg.channel_id.say(&ctx.http, response.as_ref()).await;
            }
            Command::Frog => {
                let from_self = *msg.author.id.as_u64() == SELF_USER_ID;
                if from_self {
                    return;
                }
                let image_response = match reqwest::get(RANDOM_FROG_URL).await {
                    Ok(r) => r,
                    Err(_) => return,
                };
                if image_response.status().as_u16() >= 300 {
                    let _ = msg.channel_id.say(&ctx.http, "🐸").await;
                    return;
                }
                let frog_bytes = match image_response.bytes().await {
                    Ok(b) => b,
                    Err(_) => return,
                };
                let _ = msg
                    .channel_id
                    .send_message(ctx.http, |msg| {
                        msg.content("froge");
                        msg.add_file((frog_bytes.as_ref(), "frog.jpg"));
                        msg
                    })
                    .await;
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

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        let guild = match reaction.guild_id {
            Some(g) if *g.as_u64() == KINGCORD_GUILD_ID => g,
            _ => {
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
                    .find(|role| CONSUL_ROLE_IDS.contains(role.as_u64()))
            })
            .is_some()
        {
            return;
        }
        let message = match reaction.message(&ctx.http).await {
            Ok(msg) => msg,
            Err(e) => {
                log::debug!("Failed to get message: {:?}", e);
                return;
            }
        };
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

        match member.add_role(&ctx.http, KINGCORD_TIMEOUT_ROLE_ID).await {
            Ok(_) => {
                //generate message to indicate timeout action
                let reacter_id = match reaction.user_id {
                    Some(s) => format!("{}", s),
                    None => "Someone".to_string(),
                };
                let author_id = *message.author.id.as_u64();
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

    let mut client = Client::builder(TOKEN)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
    Ok(())
}
