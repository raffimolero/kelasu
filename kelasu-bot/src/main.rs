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

#[derive(Debug, PartialEq, Eq)]
enum TeamPreference {
    Blue,
    Either,
    Red,
}

#[derive(Debug)]
struct Lobby {
    id: LobbyId,
    /// the first player is the host.
    players: Vec<UserId>,
    channel: ChannelId,
    state: LobbyState,
}

impl Lobby {
    fn new(id: LobbyId, host: UserId, channel: ChannelId) -> Self {
        Self {
            id,
            players: vec![host],
            channel,
            state: LobbyState::new(),
        }
    }

    /// asks both players which sides they prefer.
    async fn get_user_teams(
        ctx: Context<'_>,
        players: [UserId; 2],
    ) -> Result<[TeamPreference; 2], serenity::Error> {
        let reply = ctx
            .send(|m| {
                m.content(format!(
                    "<@{}> <@{}>\nWhich sides would you like to be on?\n||You can change sides if the interaction 'fails', but don't worry, your previous preference is recorded until the game begins.||",
                    players[0], players[1]
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
                                .style(serenity::ButtonStyle::Secondary)
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

        let message = reply.message().await?;

        let mut prefs = [None, None];
        loop {
            if let [Some(a), Some(b)] = prefs {
                reply.delete(ctx).await?;
                return Ok([a, b]);
            }

            let Some(interaction) = &message
                .await_component_interaction(ctx.discord())
                .filter(move |interaction| players.contains(&interaction.user.id))
                .await else {
                    ctx.say("You didn't interact in time. Your preference has been set to 'Either'.").await?;
                    for p in prefs.iter_mut().filter(|p| p.is_none()) {
                        *p = Some(TeamPreference::Either)
                    }
                    continue;
                };

            let pref = match interaction.data.custom_id.as_str() {
                "blue" => TeamPreference::Blue,
                "either" => TeamPreference::Either,
                "red" => TeamPreference::Red,
                other => {
                    eprintln!("Unknown button id: {other:?}");
                    TeamPreference::Either
                }
            };
            let this = (players[1] == interaction.user.id) as usize;
            prefs[this] = Some(pref);
        }
    }

    async fn start(&mut self, ctx: Context<'_>) -> Result<(), serenity::Error> {
        use TeamPreference::*;
        let mut pair = [self.players[0], self.players[1]];

        if match Self::get_user_teams(ctx, pair).await? {
            [Either, Either] | [Blue, Blue] | [Red, Red] => rand::random(),
            [Red | Either, Blue | Either] => true,
            [Blue | Either, Red | Either] => false,
        } {
            pair.swap(0, 1);
        }

        let game = Game::new(pair[0], pair[1]);
        ctx.channel_id()
            .say(
                &ctx.discord().http,
                format!(
                    "Game starting!\n\
                    Lobby: {}\n\
                    Blue: <@{}>,\n\
                    Red: <@{}>.\n\
                    Good luck, have fun!",
                    self.id, game.blue, game.red,
                ),
            )
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
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(Data::new()) }));

    framework.run().await.unwrap();
}
