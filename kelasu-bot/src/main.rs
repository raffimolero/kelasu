use std::{collections::HashMap, fmt::Display};

use poise::serenity_prelude::{self as serenity, ChannelId, Mutex, UserId};

type LobbyId = String;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
// User data, which is stored and accessible in all command invocations
struct Data {
    numbers: Mutex<Vec<i32>>,
    lobbies: Mutex<HashMap<LobbyId, Lobby>>,
}

#[derive(PartialEq, Eq)]
enum TeamPreference {
    Blue,
    Either,
    Red,
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

    async fn get_user_team(
        ctx: Context<'_>,
        user_id: UserId,
    ) -> Result<TeamPreference, serenity::Error> {
        // ctx.discord().http.get_user(user_id.0).await?.;
        let reply = ctx
            .send(|m| {
                m.content(format!(
                    "@{} Which side would you like to be on?",
                    user_id.0
                ))
                .components(|c| {
                    c.create_action_row(|r| {
                        r.create_button(|b| {
                            b.custom_id("blue")
                                .label("Blue")
                                .style(serenity::ButtonStyle::Primary)
                        })
                        .create_button(|b| {
                            b.custom_id("either")
                                .label("Either")
                                .style(serenity::ButtonStyle::Unknown)
                        })
                        .create_button(|b| {
                            b.custom_id("red")
                                .label("Red")
                                .style(serenity::ButtonStyle::Danger)
                        })
                    })
                })
            })
            .await?;

        let interaction = reply
            .message()
            .await?
            .await_component_interaction(ctx.discord())
            .author_id(user_id)
            .await;

        reply.delete(ctx).await?;
        let pressed_button_id = match &interaction {
            Some(m) => &m.data.custom_id,
            None => {
                ctx.say("You didn't interact in time").await?;
                return Ok(TeamPreference::Either);
            }
        };

        Ok(match &**pressed_button_id {
            "blue" => TeamPreference::Blue,
            "either" => TeamPreference::Either,
            "red" => TeamPreference::Red,
            other => {
                eprintln!("Unknown button id: {other:?}");
                TeamPreference::Either
            }
        })
    }

    async fn start(&mut self, ctx: Context<'_>) -> Result<(), serenity::Error> {
        use TeamPreference::*;
        let mut pair = [self.players[0], self.players[1]];
        if match tokio::try_join!(
            Self::get_user_team(ctx, pair[0]),
            Self::get_user_team(ctx, pair[1]),
        )? {
            (Either, Either) | (Blue, Blue) | (Red, Red) => rand::random(),
            (Red | Either, Blue | Either) => true,
            (Blue | Either, Red | Either) => false,
        } {
            pair.swap(0, 1);
        }
        let game = Game::new(pair[0], pair[1]);
        ctx.say(format!(
            "Game starting!\nBlue: @{},\nRed: @{}.",
            game.blue, game.red,
        ))
        .await?;
        self.state = LobbyState::Ongoing(game);
        Ok(())
    }
}

#[derive(Debug)]
struct Game {
    blue: UserId,
    red: UserId,
}

impl Game {
    fn new(blue: UserId, red: UserId) -> Self {
        Self { blue, red }
    }
}

#[derive(Debug)]
enum LobbyState {
    Waiting,
    Ongoing(Game),
}

impl LobbyState {
    fn new() -> Self {
        Self::Waiting
    }

    fn is_open(&self) -> bool {
        matches!(self, LobbyState::Waiting)
    }

    fn is_closed(&self) -> bool {
        !self.is_open()
    }
}

impl Display for LobbyState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LobbyState::Waiting => write!(f, "Waiting for opponent..."),
            LobbyState::Ongoing(..) => write!(f, "Ongoing match."),
        }
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
        .await
        .iter()
        .map(|(k, v)| format!("Name: `{}` ({})\n", k, v.state))
        .collect::<String>();
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
        "That lobby already exists."
    } else {
        lobbies.insert(name, Lobby::new(ctx.author().id, ctx.channel_id()));
        "Created lobby."
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
    let response = 'a: {
        let mut lobbies = ctx.data().lobbies.lock().await;
        let Some(lobby) = lobbies.get_mut(&name) else {
            break 'a "That lobby does not exist.";
        };
        let player = ctx.author().id;
        if lobby.players.contains(&player) {
            break 'a "You cannot join the same lobby twice.";
        }
        if lobby.state.is_closed() {
            break 'a "That lobby is no longer accepting players.";
        }
        lobby.players.push(player);
        lobby.start(ctx).await?;
        "Successfully joined lobby."
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
            commands: vec![age(), host(), join(), lobbies(), register()],
            ..Default::default()
        })
        .token(dotenv::var("BOT_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(serenity::GatewayIntents::non_privileged())
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(Data::new()) }));

    framework.run().await.unwrap();
}
