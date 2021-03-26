use chrono::{offset::Utc, DateTime, Datelike, Duration, Weekday};
use rand::{thread_rng, Rng};
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use uwuifier;

mod command;
use command::{Command, FRIDAY_GIFS, QUOTES};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        match msg.content.parse::<Command>() {
            Ok(command) => {
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
                };
            }
            Err(_) => {
                let i = {
                    let mut rng = thread_rng();
                    rng.gen_range(0..200)
                };
                if i == 200
                    || msg.content == "hello i would like to be uwuified please" && !msg.author.bot
                {
                    let uwuified = uwuifier::uwuify_str_sse(msg.content.as_str());
                    let _ = msg.channel_id.say(&ctx.http, uwuified).await;
                }
            }
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
