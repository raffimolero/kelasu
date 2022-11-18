use std::{collections::HashMap, sync::Mutex};

use poise::serenity_prelude::{self as serenity, ChannelId, UserId};

type LobbyId = String;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
// User data, which is stored and accessible in all command invocations
struct Data {
    numbers: Mutex<Vec<i32>>,
    lobbies: Mutex<HashMap<LobbyId, Lobby>>,
}

#[derive(Debug)]
struct Lobby {
    /// the first player is the host.
    players: Vec<UserId>,
    channel: ChannelId,
    state: LobbyState,
}

impl Lobby {
    fn new(host: UserId, channel: ChannelId) -> Self {
        Self {
            players: vec![host],
            channel,
            state: LobbyState::new(),
        }
    }
}

#[derive(Debug)]
enum LobbyState {
    Waiting,
    Ongoing,
}

impl LobbyState {
    fn new() -> Self {
        Self::Waiting
    }
}

impl Data {
    fn new() -> Self {
        Self {
            numbers: Mutex::from(vec![]),
            lobbies: Mutex::new(HashMap::new()),
        }
    }
}

/// lists all active lobbies.
#[poise::command(slash_command, prefix_command)]
async fn lobbies(ctx: Context<'_>) -> Result<(), Error> {
    let response = ctx
        .data()
        .lobbies
        .lock()
        .map_or("Couldn't list lobbies.".to_owned(), |lobbies| {
            format!("{lobbies:#?}")
        })
        .to_string();
    ctx.say(response).await?;
    Ok(())
}

/// Creates a new lobby for other players to join.
#[poise::command(slash_command, prefix_command)]
async fn host(
    ctx: Context<'_>,
    #[description = "The name of the new lobby."] name: String,
) -> Result<(), Error> {
    let response = if let Ok(mut lobbies) = ctx.data().lobbies.lock() {
        if lobbies.contains_key(&name) {
            "That lobby already exists."
        } else {
            lobbies.insert(name, Lobby::new(ctx.author().id, ctx.channel_id()));
            "Created lobby."
        }
    } else {
        "Failed."
    };
    ctx.say(response).await?;
    Ok(())
}

// TEMP
/// adds a number to the existing list of numbers
#[poise::command(slash_command, prefix_command)]
async fn number(
    ctx: Context<'_>,
    #[description = "The new number to add"] num: i32,
) -> Result<(), Error> {
    let response = if let Ok(mut numbers) = ctx.data().numbers.lock() {
        numbers.push(num);
        format!("added {num} to numbers.\nNumbers is now {numbers:#?}.")
    } else {
        "failed.".to_owned()
    };
    ctx.say(response).await?;
    Ok(())
}

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(prefix_command)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![age(), number(), host(), lobbies(), register()],
            ..Default::default()
        })
        .token(dotenv::var("BOT_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(serenity::GatewayIntents::non_privileged())
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(Data::new()) }));

    framework.run().await.unwrap();
}
