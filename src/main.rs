use chrono::{Datelike, Local, Weekday};
use rand::{thread_rng, Rng};
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

mod command;
use command::{Command, QUOTES, FRIDAY_GIFS};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if let Ok(command) = msg.content.parse::<Command>() {
            match command {
                Command::Salt => {
                    let response = {
                        let mut rng = thread_rng();
                        let quote = QUOTES[rng.gen_range(0, QUOTES.len())];
                        format!("```py\n'''\n{}\n'''```", quote)
                    };
                    //ignore error
                    let _ = msg.channel_id.say(&ctx.http, response).await;
                }
                Command::Friday => {
                    let now = Local::now();
                    let weekday = now.weekday();
                    let response = match weekday {
                        Weekday::Fri => {
                            let mut rng = thread_rng();
                            let quote = FRIDAY_GIFS[rng.gen_range(0, FRIDAY_GIFS.len())];
                            format!("it's motha fucken FRIDAY!!\n{}", quote)
                        }
                        _ => "it is not friday".to_string()
                    };
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
    const TOKEN: &'static str = include_str!("token.txt");

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
