use rand::{Rng, thread_rng};
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use tokio::prelude::*;

struct Handler {
    quotes: Box<[&'static str]>,
}

impl Handler {
    fn new() -> Self {
        Self {
            quotes: Box::new([
                "Complaining is saying \"chain grabs are fucking stupid\" which they aren't. \
                They're just incredibly braindead",
                "labbing is shit",
                "blocking is cowardly",
                "Enjoy Your Life In Hell Motherfucken Cheater You Must Really Enjoy Kissing \
                The Devil's Ass All You Did Was Stand There Cowardly Blocking An Doing The Same \
                Attack Over An Over Again You Must Really Want To Go To Hell Just To Win Over A \
                Dumb Game Well Have Fun Being Torture In Hell",
                "You're good in a dead game that no one will remember. Try doing it in a game \
                well respected in the competitive FPS community and I guarantee you won't last a year.",
                "he threw me 3 times in a row.\nthat merits a rage quit.",
                "Fighting games are stupid, unbalanced, not at all fun, and should have died in the 90s",
                "minus is unsafe",
                "Can't you just play without using exploits that the developers clearly \
                didn't thought about when making the rules?",
                "you only know how to grab and use the same move ur not good bud",
                "as long as you promise not to taunt, play too patiently to get under my skin, not \
                to have bad manners and not pick ken, necro, gouki, makoto, chun, yun, yang, oro, \
                ibuki or elena then yes\nwe can play",
                "Hot Take: You Arent Good If You Use a Grappler\ngetting punched in the face 300 times and \
                then calling heads on two coin flips with your last 10 hp does not make you good bro",
                "whoever made the first fighting game with a character built around spamming \
                and then calling it \"zoning\" and turned it into some cool archetype deserves to \
                be drawn and quartered in public",
                "This gigas is literally everything wrong with tekken atm,combos are such a crutch \
                because they give too much reward, so durrr launcher go brrr is actually really \
                a legitimate strategy",
                "losing doesn't make you a loser, wining unfairly does",
                "And i say it again all high tier players are assholes",
                "But telling me to go into the lab\nIs not helping me learn\nthat is being an asshole and \
                not wanting to help me",
                "1: why do you get launched\n2: Because the grab misses\n1: why does it miss\n2: because \
                It's never close enough\n1: why aren't you close enough\n2: I dunno",
                "I AM SO DONE WITH THIS GARBAGE ASS GAME\nAND FUCK YOU TOO",
                "I hate playin the Ditto, because it just comes down to better decision making",
                "1: i told you to block on wakeup and you said it \"doesn't work\"\n1: I also told \
                you you have an invincible dp\n2: It doesn;t\n2: His dp is ass\n1: His dp is invincible\n\
                2: It's ass",
                "If you're mad because I 'plugged' or whatever, chances are that you were: scrumming, \
                spamming, cowarding, a Paul player or just too boring a player to deal with. I will \
                always stay if it's a decent match, win or lose, but resort to either of the previous \
                and I don't mind taking a hit to the rank, it means very little to me.",
            ]),
        }
    }
}

// const SALTY_QUOTES: [&'static str; 21] =
// [
//     "Complaining is saying \"chain grabs are fucking stupid\" which they aren't. They're just incredibly braindead",
//     "labbing is shit",
//     "blocking is cowardly",
//     "Enjoy Your Life In Hell Motherfucken Cheater You Must Really Enjoy Kissing \
//     The Devil's Ass All You Did Was Stand There Cowardly Blocking An Doing The Same \
//     Attack Over An Over Again You Must Really Want To Go To Hell Just To Win Over A \
//     Dumb Game Well Have Fun Being Torture In Hell",
//     "You're good in a dead game that no one will remember. Try doing it in a game \
//     well respected in the competitive FPS community and I guarantee you won't last a year.",
//     "he threw me 3 times in a row.\nthat merits a rage quit.",
//     "Fighting games are stupid, unbalanced, not at all fun, and should have died in the 90s",
//     "minus is unsafe",
//     "Can't you just play without using exploits that the developers clearly \
//     didn't thought about when making the rules?",
//     "you only know how to grab and use the same move ur not good bud",
//     "as long as you promise not to taunt, play too patiently to get under my skin, not \
//     to have bad manners and not pick ken, necro, gouki, makoto, chun, yun, yang, oro, \
//     ibuki or elena then yes\nwe can play",
//     "Hot Take: You Arent Good If You Use a Grappler\ngettign punched in the face 300 times and \
//     then calling heads on two coin flips with your last 10 hp does not make you good bro",
//     "whoever made the first fighting game with a character built around spamming \
//     and then calling it \"zoning\" and turned it into some cool archetype deserves to \
//     be drawn and quartered in public",
//     "This gigas is literally everything wrong with tekken atm,combos are such a crutch \
//     because they give too much reward, so durrr launcher go brrr is actually really \
//     a legitimate strategy",
//     "losing doesn't make you a loser, wining unfairly does",
//     "And i say it again all high tier players are assholes",
//     "But telling me to go into the lab\nIs not helping me learn\nthat is being an asshole and \
//     not wanting to help me",
//     "1: why do you get launched\n2: Because the grab misses\n1: why does it miss\n2: because \
//     It's never close enough\n1: why aren't you close enough\n2: I dunno",
//     "I AM SO DONE WITH THIS GARBAGE ASS GAME\nAND FUCK YOU TOO",
//     "I hate playin the Ditto, because it just comes down to better decision making",
//     "1: i told you to block on wakeup and you said it \"doesn't work\"\n1: I also told \
//     you you have an invincible dp\n2: It doesn;t\n2: His dp is ass\n1: His dp is invincible\n\
//     2: It's ass",
// ];

#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "-salt" {
            let response = {
                let mut rng = thread_rng();
                format!("```apache\n{}```", self.quotes[rng.gen_range(0, self.quotes.len())])
            };
            if let Err(why) = msg.channel_id.say(&ctx.http, response).await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
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
        .event_handler(Handler::new())
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
