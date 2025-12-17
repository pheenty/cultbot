use aho_corasick::AhoCorasick;
use once_cell::sync::Lazy;
use rand::Rng;
use serenity::{async_trait, model::channel::Message, prelude::{Client, Context, EventHandler, GatewayIntents}};
use std::{env, fs, sync::atomic::{AtomicU64, Ordering}, time::{SystemTime, UNIX_EPOCH}};

struct Handler;

static CHANCE: Lazy<f64> = Lazy::new(|| {
    let chance = env::var("CHANCE")
        .unwrap_or_else(|_| {
            println!("WARNING: expected CHANCE in the environment, but it was not found. Falling back to 10%.");
            return "10".to_owned();
        })
        .parse::<f64>()
        .expect("ERROR: Failed to parse CHANCE. Is it a number?");

    if !(0_f64..=100_f64).contains(&chance) {
        panic!("ERROR: CHANCE should be a percentage, 0 through 100, but parsed {chance}");
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

    let lore: Vec<String> = fs::read_to_string(path)
        .expect("ERROR: Failed to read {path}")
        .split(splitter).map(|string| string.trim().to_owned()).collect();

    if lore.is_empty() {
        panic!("ERROR: lore was read, but happens to be empty");
    }

    lore
});

static TRIGGER_AC: Lazy<AhoCorasick> = Lazy::new(|| {
    let splitter: char = env::var("TRIGGER_SPLITTER")
        .unwrap_or_else(|_| {
            println!("WARNING: expected TRIGGER_SPLITTER in the environment, but it was not found. Falling back to \"~\".");
            "~".to_owned()
        })
        .trim().parse()
        .expect("ERROR: Failed to parse TRIGGER_SPLITTER. Is it a character?");

    let triggers: Vec<String> = env::var("TRIGGERS")
        .expect("ERROR: Expected TRIGGERS in the environment")
        .split(splitter).map(|string| string.trim().to_lowercase()).collect();

    AhoCorasick::new(triggers) // BLAZING FAST 🚀🚀🚀
        .expect("ERROR: Failed to initialize Aho-Corasick for triggers")
});

static DISABLE_AC: Lazy<AhoCorasick> = Lazy::new(|| {
    let splitter: char = env::var("DISABLE_SPLITTER")
        .unwrap_or_else(|_| {
            println!("WARNING: expected DISABLE_SPLITTER in the environment, but it was not found. Falling back to \"~\".");
            "~".to_owned()
        })
        .trim().parse()
        .expect("ERROR: Failed to parse DISABLE_SPLITTER. Is it a character?");

    let triggers: Vec<String> = env::var("DISABLERS")
        .expect("ERROR: Expected DISABLERS in the environment")
        .split(splitter).map(|string| string.trim().to_lowercase()).collect();

    AhoCorasick::new(triggers) // BLAZING FAST 🚀🚀🚀
        .expect("ERROR: Failed to initialize Aho-Corasick for disablers")
});

static DISABLED_UNTIL: AtomicU64 = AtomicU64::new(0);

static DISABLE_FOR: Lazy<u64> = Lazy::new(|| {
    env::var("DISABLE_FOR")
        .unwrap_or_else(|_| {
            println!("WARNING: expected DISABLE_FOR in the enviroment, but it was not found. Falling back to 600.");
            "600".to_owned()
        })
        .trim().parse()
        .expect("ERROR: Failed to parse DISABLE_FOR. Is it an integer amount of seconds?")
});

fn time_now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH)
        .expect("ERROR: current time is before unix epoch. How.")
        .as_secs()
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let msg_lowercase = msg.content.to_lowercase();

        if DISABLE_AC.is_match(&msg_lowercase) {
            DISABLED_UNTIL.store(time_now() + *DISABLE_FOR, Ordering::Relaxed);
            // TODO sad reaction
            return;
        }

        if msg.author.bot
            || !rand::rng().random_bool(*CHANCE)
            || time_now() > DISABLED_UNTIL.load(Ordering::Relaxed)
            || !TRIGGER_AC.is_match(&msg_lowercase) {
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
