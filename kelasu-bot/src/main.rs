use crate::lobby::{Lobby, LobbyId};
use std::{collections::HashMap, sync::Arc};

use kelasu_game::piece::Team;
use poise::serenity_prelude::{self as serenity, RwLock};
use tracing::info;
use tracing_subscriber;

mod game;
mod lobby;
mod util;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Lobbies, Error>;

// User data, which is stored and accessible in all command invocations
pub struct Lobbies {
    // NOTE: this will be slower the more users there will be.
    // not much of a concern if it's not popular, though :P
    lobbies: RwLock<HashMap<LobbyId, Lobby>>,
}

impl Lobbies {
    fn new() -> Self {
        Self {
            lobbies: RwLock::new(HashMap::new()),
        }
    }
}

/// lists all active lobbies.
#[poise::command(slash_command, prefix_command)]
async fn lobbies(ctx: Context<'_>) -> Result<(), Error> {
    info!("{} invoked /lobbies", ctx.author().name);
    // get all lobbies
    let lobbies = ctx.data().lobbies.read().await;

    // check if empty
    if lobbies.is_empty() {
        ctx.say("There are no active lobbies...").await?;
        return Ok(());
    }

    // list all active lobbies
    let mut response = "Active lobbies:".to_owned();
    for (k, v) in lobbies.iter() {
        // list lobby name
        response.push_str(&format!("\nName: `{}`\n- Players: ", k,));

        // Host, player, player, player
        // homemade intersperse
        let mut iter = v.players.iter().map(|p| p.name.as_str());
        response.push_str(iter.next().unwrap_or("Hostless lobby???"));
        for s in iter {
            response.push_str(", ");
            response.push_str(s);
        }

        // status
        response.push_str(&format!("\n- Status: {}", v.status));
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
    info!("{} invoked /host {name}", ctx.author().name);
    let mut lobbies = ctx.data().lobbies.write().await;
    let response = if lobbies.contains_key(&name) {
        "That lobby already exists.".to_owned()
    } else {
        let id = Arc::new(name);
        lobbies.insert(id.clone(), Lobby::new(id.clone(), ctx.author().into()));
        format!("Created lobby: {id}")
    };
    info!(response);
    ctx.say(response).await?;
    Ok(())
}

async fn try_join(ctx: Context<'_>, name: &String) -> Result<bool, Error> {
    // try to pair up players
    let pair = {
        let mut lobbies = ctx.data().lobbies.write().await;
        let Some(lobby) = lobbies.get_mut(&*name) else {
            ctx.say("That lobby does not exist.").await?;
            return Ok(false);
        };
        let player = ctx.author();
        if lobby.players.iter().any(|p| p.id == player.id) {
            ctx.say("You cannot join the same lobby twice.").await?;
            return Ok(false);
        }
        if lobby.status.is_closed() {
            ctx.say("That lobby is no longer accepting players.")
                .await?;
            return Ok(false);
        }
        info!("joined {name}");
        lobby.add_player(ctx, player).await?;
        [lobby.players[0].id, lobby.players[1].id]
    }; // release the lock

    // ask both players their preferred teams
    let teams = Lobby::get_user_teams(ctx, pair).await?;

    let mut game = {
        // find the lobby again
        let mut lobbies = ctx.data().lobbies.write().await;
        let Some(lobby) = lobbies.get_mut(&*name) else {
            ctx.say(format!("This lobby ({name}) somehow no longer exists...")).await?;
            return Ok(false)
        };

        // start
        info!("starting lobby");
        lobby.start(ctx, teams).await?
    };

    let result = game.start(ctx).await?;
    let result = match result.0 {
        Some(Team::Blue) => format!("<@{}> won against <@{}>!", game.blue, game.red),
        Some(Team::Red) => format!("<@{}> won against <@{}>!", game.red, game.blue),
        None => format!("Draw between <@{}> and <@{}>!", game.blue, game.red),
    };
    ctx.say(format!("Game over!\nResult: {result}")).await?;

    Ok(true)
}

/// Joins a lobby.
#[poise::command(slash_command, prefix_command)]
async fn join(
    ctx: Context<'_>,
    #[description = "The name of the lobby to join."] name: String,
) -> Result<(), Error> {
    info!("{} invoked /join {name}", ctx.author().name);
    match &try_join(ctx, &name).await {
        // don't close if not requested
        Ok(false) => return Ok(()),
        // close if requested
        Ok(true) => {}
        // close on error
        Err(e) => {
            ctx.say(format!("Error: {e}")).await?;
        }
    }
    let mut lobbies = ctx.data().lobbies.write().await;
    if let Some(_lobby) = lobbies.remove(&name) {
        info!("Closed lobby {name}");
        ctx.say(format!("Closed lobby: `{name}`.")).await?;
    } else {
        info!("tried to delete lobby {name}");
        ctx.say(format!("The lobby `{name}` just disappeared?"))
            .await?;
    }
    Ok(())
}

#[poise::command(prefix_command)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![host(), join(), lobbies(), register()],
            ..Default::default()
        })
        .token(dotenv::var("BOT_TOKEN").expect("missing BOT_TOKEN"))
        .intents(serenity::GatewayIntents::non_privileged())
        .user_data_setup(move |_ctx, _ready, _framework| {
            Box::pin(async move { Ok(Lobbies::new()) })
        });

    framework.run().await.unwrap();
}
