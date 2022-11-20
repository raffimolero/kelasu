use crate::{
    game::{Game, TeamPreference},
    util::respond_ephemeral,
    Context,
};
use std::fmt::Display;

use poise::serenity_prelude::{self as serenity, User, UserId};

pub type LobbyId = String;

#[derive(Debug)]
pub enum LobbyStatus {
    Waiting,
    Starting,
    Ongoing,
}

impl LobbyStatus {
    pub fn new() -> Self {
        Self::Waiting
    }

    pub fn is_open(&self) -> bool {
        matches!(self, LobbyStatus::Waiting)
    }

    pub fn is_closed(&self) -> bool {
        !self.is_open()
    }
}

impl Display for LobbyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LobbyStatus::Waiting => write!(f, "Waiting for opponent..."),
            LobbyStatus::Starting => write!(f, "Starting game..."),
            LobbyStatus::Ongoing => write!(f, "Ongoing match..."),
        }
    }
}

#[derive(Debug)]
pub struct UserInfo {
    pub id: UserId,
    pub name: String,
}

impl From<&User> for UserInfo {
    fn from(u: &User) -> Self {
        Self {
            id: u.id,
            name: u.name.clone(),
        }
    }
}

#[derive(Debug)]
pub struct Lobby {
    pub id: LobbyId,
    /// the first player is the host.
    pub players: Vec<UserInfo>,
    pub status: LobbyStatus,
}

impl Lobby {
    pub fn new(id: LobbyId, host: UserInfo) -> Self {
        Self {
            id,
            players: vec![host],
            status: LobbyStatus::new(),
        }
    }

    /// asks both players which sides they prefer.
    pub async fn get_user_teams(
        ctx: Context<'_>,
        players: [UserId; 2],
    ) -> Result<[TeamPreference; 2], serenity::Error> {
        let reply = ctx
            .send(|m| {
                m.content(format!(
                    "Starting game!\n\
                    <@{}> <@{}>\n\
                    Which sides would you like to be on? You can change sides.",
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
        while prefs.contains(&None) {
            let Some(interaction) = &message
                .await_component_interaction(ctx.discord())
                .await
            else {
                ctx.say(format!(
                    "{}{}You didn't interact in time. Your preference has been set to 'Either'.",
                    if prefs[0].is_none() { format!("<@{}> ", players[0].0) } else { "".to_owned() },
                    if prefs[1].is_none() { format!("<@{}> ", players[1].0) } else { "".to_owned() },
                )).await?;
                break;
            };

            if !players.contains(&interaction.user.id) {
                respond_ephemeral(ctx, interaction, "You are not in that lobby.").await?;
                continue;
            }

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
        reply.delete(ctx).await?;

        Ok(prefs.map(|p| p.unwrap_or_default()))
    }

    pub async fn start(
        &mut self,
        ctx: Context<'_>,
        teams: [TeamPreference; 2],
    ) -> Result<Game, serenity::Error> {
        use TeamPreference::*;
        let mut pair = [self.players[0].id, self.players[1].id];

        if match teams {
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

        self.status = LobbyStatus::Ongoing;
        Ok(game)
    }
}
