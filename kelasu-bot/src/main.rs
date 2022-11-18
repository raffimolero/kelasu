use crate::lobby::{Lobby, LobbyId};
use std::collections::HashMap;

use poise::serenity_prelude::{self as serenity, Mutex};

mod game;
mod lobby;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Lobbies, Error>;

// User data, which is stored and accessible in all command invocations
pub struct Lobbies {
    lobbies: Mutex<HashMap<LobbyId, Lobby>>,
}

impl Lobbies {
    fn new() -> Self {
        Self {
            lobbies: Mutex::new(HashMap::new()),
        }
    }
}

/// lists all active lobbies.
#[poise::command(slash_command, prefix_command)]
async fn lobbies(ctx: Context<'_>) -> Result<(), Error> {
    let lobbies = ctx.data().lobbies.lock().await;
    if lobbies.is_empty() {
        ctx.say("There are no active lobbies...").await?;
        return Ok(());
    }
    let mut response = "Active lobbies:".to_owned();
    for (k, v) in lobbies.iter() {
        response.push_str(&format!("\nName: `{}` ({})", k, v.state));
    }
    ctx.say(response).await?;
    Ok(())
}

/// Creates a new lobby for other players to join.
#[poise::command(slash_command, prefix_command)]
async fn host(
    ctx: Context<'_>,
    #[description = "The name of the new lobby."] name: String,
) -> Result<(), Error> {
    let mut lobbies = ctx.data().lobbies.lock().await;
    let response = if lobbies.contains_key(&name) {
        "That lobby already exists.".to_owned()
    } else {
        lobbies.insert(
            name.clone(),
            Lobby::new(name.clone(), ctx.author().id, ctx.channel_id()),
        );
        format!("Created lobby: {name}")
    };
    ctx.say(response).await?;
    Ok(())
}

/// Joins a lobby.
#[poise::command(slash_command, prefix_command)]
async fn join(
    ctx: Context<'_>,
    #[description = "The name of the lobby to join."] name: String,
) -> Result<(), Error> {
    let mut lobbies = ctx.data().lobbies.lock().await;
    let Some(lobby) = lobbies.get_mut(&name) else {
        ctx.say("That lobby does not exist.").await?;
        return Ok(());
    };
    let player = ctx.author().id;
    if lobby.players.contains(&player) {
        ctx.say("You cannot join the same lobby twice.").await?;
        return Ok(());
    }
    if lobby.state.is_closed() {
        ctx.say("That lobby is no longer accepting players.")
            .await?;
        return Ok(());
    }
    lobby.players.push(player);
    lobby.start(ctx).await?;
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
            commands: vec![host(), join(), lobbies(), register()],
            ..Default::default()
        })
        .token(dotenv::var("BOT_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(serenity::GatewayIntents::non_privileged())
        .user_data_setup(move |_ctx, _ready, _framework| {
            Box::pin(async move { Ok(Lobbies::new()) })
        });

    framework.run().await.unwrap();
}
