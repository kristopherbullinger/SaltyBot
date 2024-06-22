use arrayvec::ArrayVec;
use snafu::{OptionExt, Snafu};
use std::borrow::Cow;
use std::convert::TryFrom;
use std::fmt::Write;
use std::sync::Mutex;

use chrono::{offset::Utc, DateTime, Datelike, Duration, Weekday};
use rand::{thread_rng, Rng};
use serenity::{
    async_trait,
    model::{
        channel::{Message, Reaction, ReactionType},
        gateway::{GatewayIntents, Ready},
        timestamp::Timestamp,
    },
    prelude::*,
};

mod command;
mod glossary;
mod utils;
use command::{Command, QUOTES};

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

const KINGCORD_GUILD_ID: u64 = 350242625502052352;
const SELF_USER_ID: u64 = 751611106107064451;
const SPEEZ_USER_ID: u64 = 442321800416854037;
static CONSUL_ROLE_IDS: &'static [u64] = &[
    350362647989846026, //admin
    432017127810269204, //moderator
    885971978052325376, //council
];
static NECO_ARC_DOUGIE: &str = "https://cdn.discordapp.com/attachments/350242625502052353/1010292204201332778/EynKWlUtroS3hAf4.mp4";
static NECO_ARC_SMOKING: &str = "https://pbs.twimg.com/media/FE6QLYLXEAg-ccT.jpg";
static NECO_ARC_SEATBELT: &str = "https://cdn.discordapp.com/attachments/350242625502052353/1090717976765931530/20230329_104015.png";

struct Handler {
    unsplash_client_id: String,
}

const ONE_DAY: i64 = 24 * 60 * 60;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let content = msg.content.as_str();
        //if message is profane and sent in kingcord, silence user
        if msg.guild_id.map(|g| g.0) == Some(KINGCORD_GUILD_ID) && utils::is_profane(content) {
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
                    Weekday::Fri => NECO_ARC_DOUGIE,
                    Weekday::Sat if msg.author.id.0 == SPEEZ_USER_ID => NECO_ARC_SMOKING,
                    _ => NECO_ARC_SEATBELT,
                };
                let _ = msg.channel_id.say(&ctx.http, response).await;
            }
            Command::Frog => {
                let from_self = *msg.author.id.as_u64() == SELF_USER_ID;
                if from_self {
                    return;
                }
                let frog_img = match random_frog_img(&self.unsplash_client_id).await {
                    Ok(img) => img,
                    Err(e) => {
                        let _ = msg.channel_id.say(&ctx.http, format!("🐸: {:?}", e)).await;
                        return;
                    }
                };

                for text in frog_img.messages() {
                    let _ = msg.channel_id.say(&ctx.http, text).await;
                }
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
    dotenvy::dotenv().ok();
    env_logger::init();
    glossary::init()?;
    let discord_token = std::env::var("DISCORD_TOKEN")?;
    let unsplash_client_id = std::env::var("UNSPLASH_CLIENT_ID")?;

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(discord_token, intents)
        .event_handler(Handler { unsplash_client_id })
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
    Ok(())
}
async fn random_frog_img(unsplash_client_id: &str) -> Result<FrogImg, Error> {
    // store up to 30 urls to reduce API usage
    static FROG_IMG_URLS: Mutex<ArrayVec<FrogImg, 30>> = Mutex::new(ArrayVec::new_const());
    let mut v = FROG_IMG_URLS.lock().unwrap().take();
    match v.pop() {
        Some(url) => Ok(url),
        None => {
            let cl = reqwest::Client::new();
            let basic_auth_val = format!("Client-ID {}", unsplash_client_id);
            let resp = cl
                .get("https://api.unsplash.com/photos/random?query=frog&count=30")
                .header("Authorization", basic_auth_val)
                .send()
                .await?
                .error_for_status()?;
            let body = resp.json::<Vec<unsplash::GetRandomImageResponse>>().await?;
            let mut imgs = body.into_iter();
            let frogimg = imgs.next().context(NoUrlFoundSnafu)?.into();
            v.extend(imgs.map(Into::into).take(30));
            *FROG_IMG_URLS.lock().unwrap() = v;
            Ok(frogimg)
        }
    }
}

#[derive(Debug, Snafu)]
#[snafu(display("no urls found in response body"))]
struct NoUrlFoundError;

struct FrogImg {
    img_url: String,
    photographer_name: Option<String>,
    photographer_portfolio_url: Option<String>,
}

impl FrogImg {
    fn messages(&self) -> [String; 2] {
        let mut msg1 = String::new();
        let _ = write!(&mut msg1, "froge\n{}", self.img_url);
        let mut msg2 = String::new();
        match (&self.photographer_name, &self.photographer_portfolio_url) {
            (Some(name), Some(port)) => {
                let _ = write!(&mut msg2, "\nPhoto by: [{}]({}) on [Unsplash](https://unsplash.com/?utm_source=frogbot&utm_medium=referral)", name, port);
            }
            (Some(name), None) => {
                let _ = write!(&mut msg2, "\nPhoto by: {} on [Unsplash](https://unsplash.com/?utm_source=frogbot&utm_medium=referral)", name);
            }
            (None, Some(port)) => {
                let _ = write!(&mut msg2, "\nPhoto by: [Unknown]({}) on [Unsplash](https://unsplash.com/?utm_source=frogbot&utm_medium=referral)", port);
            }
            _ => {}
        }
        [msg1, msg2]
    }
}

impl From<unsplash::GetRandomImageResponse> for FrogImg {
    fn from(mut resp: unsplash::GetRandomImageResponse) -> FrogImg {
        FrogImg {
            img_url: resp.urls.regular,
            photographer_name: resp.user.as_mut().map(|usr| std::mem::take(&mut usr.name)),
            photographer_portfolio_url: resp
                .user
                .as_mut()
                .map(|usr| std::mem::take(&mut usr.links.html)),
        }
    }
}
mod unsplash {
    use serde::Deserialize;
    use serde::Serialize;

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct GetRandomImageResponse {
        pub urls: Urls,
        // pub links: Option<Links>,
        pub user: Option<User>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Urls {
        // pub raw: String,
        // pub full: String,
        pub regular: String,
        // pub small: String,
        // pub thumb: String,
    }

    // #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    // pub struct Links {
    //     #[serde(rename = "self")]
    //     pub self_: String,
    //     pub html: String,
    //     pub download: String,
    //     pub download_location: String,
    // }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct User {
        // pub id: String,
        // pub updated_at: String,
        // pub username: String,
        pub name: String,
        // pub portfolio_url: Option<String>,
        // pub bio: String,
        // pub location: String,
        // pub total_likes: i64,
        // pub total_photos: i64,
        // pub total_collections: i64,
        // pub instagram_username: String,
        // pub twitter_username: String,
        pub links: Links2,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Links2 {
        // #[serde(rename = "self")]
        // pub self_: String,
        pub html: String,
        // pub photos: String,
        // pub likes: String,
        // pub portfolio: String,
    }
}
