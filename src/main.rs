use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::{Arc, Mutex};

use chrono::{offset::Utc, DateTime, Datelike, Duration, Weekday};
use image::{codecs::jpeg::JpegEncoder, ImageFormat, Rgba};
use imageproc::drawing::draw_text;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use rand::{thread_rng, Rng};
use rusttype::{Font, Scale};
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready, id::GuildId},
    prelude::*,
};

mod command;
mod utils;
use command::{Command, FRIDAY_GIFS, QUOTES};

static _SILENCE_CRAB_BYTES: &[u8] = include_bytes!("../imgs/SILENCE.jpg");
static _FONTDATA: &[u8] = include_bytes!("../fonts/Ubuntu-B.ttf");
const _WHITE: Rgba<u8> = Rgba([255; 4]);
const KINGCORD_GUILD_ID: u64 = 350242625502052352;
const KINGCORD_TIMEOUT_ROLE_ID: u64 = 547814221325271072;
static RANDOM_FROG_URL: &str = "https://source.unsplash.com/450x400/?frog";

/*
struct Handler {
    //font: rusttype::Font<'static>,
    crab_timer: Arc<Mutex<HashMap<GuildId, DateTime<Utc>>>>,
}
*/
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
        if let Ok(command) = Command::try_from(content) {
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
                    //Texas is UTC-6
                    let texas_utc_offset = Duration::hours(6);
                    let texas_time = now - texas_utc_offset;
                    let weekday = texas_time.weekday();
                    let response = match weekday {
                        Weekday::Fri => {
                            let mut rng = thread_rng();
                            let quote = FRIDAY_GIFS[rng.gen_range(0..FRIDAY_GIFS.len())];
                            format!("it's motha fucken FRIDAY!!\n{}", quote)
                        }
                        _ => "it is not friday".to_string(),
                    };
                    let _ = msg.channel_id.say(&ctx.http, response).await;
                }
                //each guild may have one silence crab per 30 seconds
                Command::Silence(_silence) => {
                    let image = match reqwest::get(RANDOM_FROG_URL).await {
                        Ok(r) => r,
                        Err(_) => return,
                    };
                    let frog_bytes = match image.bytes().await {
                        Ok(b) => b,
                        Err(_) => return,
                    };
                    let _ = msg.channel_id
                        .send_message(ctx.http, |msg| {
                            msg.content("Laser Crab has entered a period of peaceful retirement. Please enjoy this picture of a frog As-salamu alaykum.");
                            msg.add_file((frog_bytes.as_ref(), "frog.jpg"));
                            msg
                        })
                        .await;
                    /*
                    // lock mutex, then check map for guild id. if found, check the timer and send waiting
                    // message if last_used less than 30 seconds ago
                    let gid = match msg.guild_id {
                        Some(gid) => gid,
                        None => return,
                    };
                    let lasers_ready = match self.crab_timer.lock() {
                        Ok(mut map) => {
                            let now = Utc::now();
                            match map
                                .get(&gid)
                                .filter(|&&last_used| (now - last_used).num_seconds() <= 30)
                            {
                                //No previous entry, or cooldown expired
                                None => {
                                    map.insert(gid, now);
                                    true
                                }
                                //previous entry and still on cooldown
                                Some(_) => false,
                            }
                        }
                        Err(_) => false,
                    };
                    if !lasers_ready {
                        let _ = msg
                            .channel_id
                            .say(&ctx.http, "Lasers cooling down...")
                            .await;
                        return;
                    }
                    //read silencecrab.jpg into memory as img file
                    let mut silencecrab_blank =
                        image::load_from_memory_with_format(SILENCE_CRAB_BYTES, ImageFormat::Jpeg)
                            .expect("Failed to read SilenceCrab");
                    //write msg mention username into the image
                    let drawn_img = draw_text(
                        &mut silencecrab_blank,
                        WHITE,
                        32,
                        120,
                        Scale::uniform(50.0),
                        &self.font,
                        silence.as_str(),
                    );
                    //encode jpg file into in-memory buffer
                    let mut buf: Vec<u8> = Vec::with_capacity(drawn_img.as_raw().len());
                    let mut encoder = JpegEncoder::new(&mut buf);
                    encoder
                        .encode_image(&drawn_img)
                        .expect("Failed to encode & save image");
                    //respond in channel with attachment
                    if let Err(why) = msg
                        .channel_id
                        .send_message(&ctx.http, |msg| {
                            msg.add_file((buf.as_slice(), "silence.jpg"))
                        })
                        .await
                    {
                        println!("{}", why);
                    }
                    */
                }
                Command::Glossary(term) => {
                    let encoded_term = utf8_percent_encode(term, NON_ALPHANUMERIC).to_string();
                    let response = format!("https://glossary.infil.net/?t={}", encoded_term);
                    let _ = msg.channel_id.say(&ctx.http, response).await;
                }
            };
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    const TOKEN: &str = include_str!("token.txt");

    /*
    let crab_timer = Arc::new(Mutex::new(HashMap::default()));
    let handler = Handler {
        font: Font::try_from_bytes(FONTDATA).expect("Failed To Parse Font Data"),
        crab_timer,
    };
    */
    // Create a new instance of the Client, logging in as a bot. This will
    // automatically prepend your bot token with "Bot ", which is a requirement
    // by Discord for bot users.
    let mut client = Client::builder(TOKEN)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
