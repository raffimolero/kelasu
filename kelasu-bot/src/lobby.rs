use crate::{
    game::{Game, TeamPreference},
    Context,
};
use std::fmt::Display;

use poise::serenity_prelude::{self as serenity, ChannelId, UserId};

pub type LobbyId = String;

#[derive(Debug)]
pub struct Lobby {
    pub id: LobbyId,
    /// the first player is the host.
    pub players: Vec<UserId>,
    pub channel: ChannelId,
    pub state: LobbyState,
}

impl Lobby {
    pub fn new(id: LobbyId, host: UserId, channel: ChannelId) -> Self {
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

    pub async fn start(&mut self, ctx: Context<'_>) -> Result<(), serenity::Error> {
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
pub enum LobbyState {
    Waiting,
    Ongoing(Game),
}

impl LobbyState {
    pub fn new() -> Self {
        Self::Waiting
    }

    pub fn is_open(&self) -> bool {
        matches!(self, LobbyState::Waiting)
    }

    pub fn is_closed(&self) -> bool {
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
