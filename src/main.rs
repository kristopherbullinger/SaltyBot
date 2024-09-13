use chrono::{offset::Utc, DateTime, Datelike, Duration, Weekday};
use serenity::{
    async_trait,
    model::{
        channel::{Message, Reaction, ReactionType},
        gateway::{GatewayIntents, Ready},
        id::ChannelId,
        timestamp::Timestamp,
    },
    prelude::*,
};
use sqlx::SqlitePool;
use std::borrow::Cow;
use std::convert::TryFrom;
use std::fmt::Write;
use std::str::FromStr;

mod command;
mod glossary;
mod utils;
use command::{AddSelfAssignRole, Command};

const KINGCORD_GUILD_ID: u64 = 350242625502052352;
const SELF_USER_ID: u64 = 751611106107064451;
const SPEEZ_USER_ID: u64 = 442321800416854037;
const MOD_ROLE_ID: u64 = 432017127810269204;
const ADMIN_ROLE_ID: u64 = 350362647989846026;
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

struct Handler {
    pool: SqlitePool,
}
const ONE_DAY: i64 = 24 * 60 * 60;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
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
            Command::RemoveSelfAssignRole(role_name) => {
                let _ = sqlx::query("DELETE FROM role_reactions WHERE role_name = ?1")
                    .bind(role_name)
                    .execute(&self.pool)
                    .await;
            }
            Command::AddSelfAssignRole(asar) => {
                if let Ok(member) = msg.member(&ctx.http).await {
                    if member
                        .roles
                        .iter()
                        .any(|role| role.get() == ADMIN_ROLE_ID || role.get() == MOD_ROLE_ID)
                    {
                        let Some(guild) = msg.guild_id else {
                            return;
                        };
                        let guild_roles = match guild.roles(&ctx.http).await {
                            Ok(r) => r,
                            Err(e) => {
                                log::error!("failed to fetch roles: {:?}", e);
                                return;
                            }
                        };
                        let Some(role) = guild_roles
                            .values()
                            .find(|role| role.name == asar.role_name)
                        else {
                            let _ = msg.channel_id.say(&ctx, "role not found");
                            return;
                        };
                        if let Err(e) = add_self_assign_role(
                            &self.pool,
                            asar.emoji,
                            asar.role_name,
                            role.id.get() as i64,
                        )
                        .await
                        {
                            let _ = msg.channel_id.say(&ctx, format!("{:?}", e));
                        }
                    }
                }
            }
            Command::ListSelfAssignRoles => {
                let _ = list_all_self_assign_roles(&ctx, &self.pool, msg.channel_id).await;
            }
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

    async fn reaction_remove(&self, ctx: Context, reaction: Reaction) {}

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
        if message.author.id.get() == SELF_USER_ID {
            if let Err(_) = handle_role_reaction(&ctx, self.pool.clone(), &reaction).await {
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

        // match member.add_role(&ctx.http, KINGCORD_TIMEOUT_ROLE_ID).await {
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

async fn list_all_self_assign_roles(
    ctx: &Context,
    db: &SqlitePool,
    channel_id: ChannelId,
) -> anyhow::Result<()> {
    let roles: Vec<Role> =
        sqlx::query_as("SELECT id, emoji, role_id, role_name FROM role_reactions LIMIT 50")
            .fetch_all(db)
            .await?;
    let mut msg = String::new();
    for role in roles {
        let _ = write!(msg, "{} | {}\n", role.emoji, role.role_name);
    }
    channel_id.say(ctx, msg).await?;
    Ok(())
}

async fn add_self_assign_role(
    db: &SqlitePool,
    emoji: &str,
    role_name: &str,
    role_id: i64,
) -> anyhow::Result<()> {
    sqlx::query("INSERT INTO role_reactions (emoji, role_id, role_name) VALUES ($1, $2, $3)")
        .bind(emoji)
        .bind(role_id)
        .bind(role_name)
        .execute(db)
        .await?;
    Ok(())
}

// async fn remove_self_assign_role(db: &SqlitePool, name: &str) -> anyhow::Result<()> {
//     sqlx::query("DELETE INTO role_reactions (emoji, role_id, role_name) VALUES ($1, $2, $3)")
//         .bind(asar.emoji)
//         .bind(asar.role_id)
//         .bind(asar.role_name)
//         .execute(db)
//         .await?;
//     Ok(())
// }

async fn handle_role_reaction(
    ctx: &Context,
    db: SqlitePool,
    reaction: &Reaction,
) -> anyhow::Result<()> {
    let emoji = match reaction.emoji {
        ReactionType::Unicode(ref e) => e,
        _ => return Ok(()),
    };
    let Some(ref member) = reaction.member else {
        return Ok(());
    };
    let role_id = match sqlx::query_as::<_, (u64,)>(
        "SELECT role_id FROM role_reactions WHERE emoji = $1 LIMIT 1",
    )
    .bind(emoji)
    .fetch_optional(&db)
    .await
    {
        Ok(Some(r)) => r.0,
        Ok(None) => return Ok(()),
        Err(e) => {
            log::error!("failed to fetch role info: {:?}", e);
            return Err(e.into());
        }
    };
    let _ = member.add_role(ctx, role_id).await?;

    Ok(())
}

#[derive(sqlx::FromRow)]
struct Role {
    id: u64,
    emoji: String,
    role_id: u64,
    role_name: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();
    glossary::init()?;

    const TOKEN: &str = include_str!("token.txt");

    let opts = sqlx::sqlite::SqliteConnectOptions::from_str("sqlite://data.db")?
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(opts).await?;
    sqlx::query(include_str!("./up.sql")).execute(&pool).await?;

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(TOKEN, intents)
        .event_handler(Handler { pool })
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
    Ok(())
}
