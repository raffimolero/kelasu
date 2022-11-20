use crate::lobby::{Lobby, LobbyId};
use std::collections::HashMap;

use kelasu_game::piece::Team;
use lobby::LobbyStatus;
use poise::serenity_prelude::{self as serenity, Mutex};

mod game;
mod lobby;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Lobbies, Error>;

// User data, which is stored and accessible in all command invocations
pub struct Lobbies {
    // NOTE: this will be slower the more users there will be.
    // not much of a concern if it's not popular, though :P
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
    // get all lobbies
    let lobbies = ctx.data().lobbies.lock().await;

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
    let mut lobbies = ctx.data().lobbies.lock().await;
    let response = if lobbies.contains_key(&name) {
        "That lobby already exists.".to_owned()
    } else {
        lobbies.insert(name.clone(), Lobby::new(name.clone(), ctx.author().into()));
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
    // try to pair up players
    let pair = {
        let mut lobbies = ctx.data().lobbies.lock().await;
        let Some(lobby) = lobbies.get_mut(&name) else {
            ctx.say("That lobby does not exist.").await?;
            return Ok(());
        };
        let player = ctx.author();
        if lobby.players.iter().any(|p| p.id == player.id) {
            ctx.say("You cannot join the same lobby twice.").await?;
            return Ok(());
        }
        if lobby.status.is_closed() {
            ctx.say("That lobby is no longer accepting players.")
                .await?;
            return Ok(());
        }
        lobby.players.push(player.into());
        lobby.status = LobbyStatus::Starting;
        [lobby.players[0].id, lobby.players[1].id]
    }; // release the lock

    // ask both players their preferred teams
    let teams = Lobby::get_user_teams(ctx, pair).await?;

    let game = {
        // find the lobby again
        let mut lobbies = ctx.data().lobbies.lock().await;
        let Some(lobby) = lobbies
        .get_mut(&name) else {
            ctx.say(format!("This lobby ({name:?}) somehow no longer exists...")).await?;
            return Ok(())
        };

        // start
        lobby.start(ctx, teams).await?
    };

    let result = game.start(ctx).await?;
    let result = match result.0 {
        Some(Team::Blue) => format!("<@{}> won against <@{}>!", pair[0].0, pair[1].0),
        Some(Team::Red) => format!("<@{}> won against <@{}>!", pair[1].0, pair[0].0),
        None => format!("Draw between <@{}> and <@{}>!", pair[0].0, pair[1].0),
    };
    ctx.say(format!("Game over!\nResult: {result}")).await?;

    // delete the lobby
    let mut lobbies = ctx.data().lobbies.lock().await;
    lobbies.remove(&name);
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
        .token(dotenv::var("BOT_TOKEN").expect("missing BOT_TOKEN"))
        .intents(serenity::GatewayIntents::non_privileged())
        .user_data_setup(move |_ctx, _ready, _framework| {
            Box::pin(async move { Ok(Lobbies::new()) })
        });

    framework.run().await.unwrap();
}
