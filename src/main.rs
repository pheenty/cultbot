use aho_corasick::AhoCorasick;
use once_cell::sync::Lazy;
use rand::Rng;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::{Client, Context, EventHandler, GatewayIntents};
use std::{env, fs};

struct Handler;

static CHANCE: Lazy<f64> = Lazy::new(|| {
    let chance = env::var("CHANCE")
        .unwrap_or_else(|_| {
            println!("WARNING: expected CHANCE in the environment, but it was not found. Falling back to 1%.");
            return "1".to_owned();
        })
        .parse::<f64>()
        .expect("ERROR: Failed to parse CHANCE. Is it a number?");

    if !(0_f64..=100_f64).contains(&chance) {
        panic!("ERROR: CHANCE should be a percentage, 0 through 100, but parsed {chance}")
    }

    chance / 100_f64
});

static LORE: Lazy<Vec<String>> = Lazy::new(|| {
    let splitter: char = env::var("LORE_SPLITTER")
        .unwrap_or_else(|_| {
            println!("WARNING: expected LORE_SPLITTER in the environment, but it was not found. Falling back to \"~\".");
            return "~".to_owned();
        })
        .trim().parse()
        .expect("ERROR: Failed to parse LORE_SPLITTER. Is it a character?");

    let path = env::var("LORE_FILE")
        .unwrap_or_else(|_| {
            println!("WARNING: expected LORE_FILE path in the environment, but it was not found. Falling back to \"/app/lore.txt\".");
            return "/app/lore.txt".to_owned();
        });

    fs::read_to_string(path)
        .expect("ERROR: Failed to read {path}")
        .split(splitter).map(|string| string.trim().to_owned()).collect()
});

static AHOCORASICK: Lazy<AhoCorasick> = Lazy::new(|| {
    let splitter: char = env::var("TRIGGER_SPLITTER")
        .unwrap_or_else(|_| {
            println!("WARNING: expected TRIGGER_SPLITTER in the environment, but it was not found. Falling back to \"~\".");
            return "~".to_owned();
        })
        .trim().parse()
        .expect("ERROR: Failed to parse TRIGGER_SPLITTER. Is it a character?");

    let triggers: Vec<String> = env::var("TRIGGERS")
        .expect("ERROR: Expected TRIGGERS in the environment")
        .split(splitter).map(|string| string.trim().to_lowercase()).collect();

    AhoCorasick::new(triggers).expect("ERROR: Failed to initialize Aho-Corasick") // BLAZING FAST 🚀🚀🚀
});

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if !rand::rng().random_bool(*CHANCE)
            || !AHOCORASICK.is_match(&msg.content.to_lowercase()) {
            return;
        }

        let random_lore = &LORE[rand::rng().random_range(0..LORE.len())];

        if let Err(why) = msg.reply(&ctx, random_lore).await {
            println!("ERROR: can't send message: {why:?}");
        }
    }
}

#[tokio::main]
async fn main() {
    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN")
        .expect("ERROR: expected DISCORD_TOKEN in the environment");

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot.
    let mut client = Client::builder(token, intents)
        .event_handler(Handler).await
        .expect("ERROR: can't create client");

    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("ERROR: client gave an error: {why:?}");
    }
}
