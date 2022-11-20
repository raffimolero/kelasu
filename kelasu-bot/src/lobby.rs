use crate::{
    game::{Game, TeamPreference},
    util::respond_ephemeral,
    Context,
};
use std::{fmt::Display, sync::Arc};

use poise::{
    futures_util::StreamExt,
    serenity_prelude::{self as serenity, User, UserId},
};

pub type LobbyId = Arc<String>;

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

    pub async fn add_player(
        &mut self,
        ctx: Context<'_>,
        player: impl Into<UserInfo>,
    ) -> Result<(), serenity::Error> {
        ctx.say(format!("Joined `{}`", self.id)).await?;

        self.players.push(player.into());
        self.status = LobbyStatus::Starting;
        Ok(())
    }

    /// asks both players which sides they prefer.
    pub async fn get_user_teams(
        ctx: Context<'_>,
        players: [UserId; 2],
    ) -> Result<[TeamPreference; 2], serenity::Error> {
        let mut prefs = [None, None];
        fn prefs_format(prefs: &[Option<TeamPreference>; 2], players: [UserId; 2]) -> String {
            format!(
                "Which sides would you like to be on? You can change sides if your opponent hasn't decided.\n\
                - <@{}>: {},\n\
                - <@{}>: {}.\n",
                players[0],
                prefs[0]
                    .as_ref()
                    .map_or("Undecided".to_owned(), |p| format!("{p:?}")),
                players[1],
                prefs[1]
                    .as_ref()
                    .map_or("Undecided".to_owned(), |p| format!("{p:?}")),
            )
        }

        let reply = ctx
            .send(|m| {
                m.content(prefs_format(&prefs, players)).components(|c| {
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

        let mut interactions = message.await_component_interactions(ctx.discord()).build();
        while prefs.contains(&None) {
            let Some(interaction) = interactions.next().await else {
                ctx.say(format!(
                    "{}{}You didn't interact in time. Your preference has been set to 'Either'.",
                    if prefs[0].is_none() { format!("<@{}> ", players[0].0) } else { "".to_owned() },
                    if prefs[1].is_none() { format!("<@{}> ", players[1].0) } else { "".to_owned() },
                )).await?;
                break;
            };
            if !players.contains(&interaction.user.id) {
                respond_ephemeral(ctx, &interaction, "You are not in that lobby.").await?;
                continue;
            }
            interaction.defer(&ctx.discord().http).await?;

            let pref = match interaction.data.custom_id.as_str() {
                "blue" => TeamPreference::Blue,
                "either" => TeamPreference::Either,
                "red" => TeamPreference::Red,
                other => {
                    eprintln!("Unknown button id: {other:?}");
                    TeamPreference::Either
                }
            };
            let player = (players[1] == interaction.user.id) as usize;
            prefs[player] = Some(pref);

            reply
                .edit(ctx, |b| b.content(prefs_format(&prefs, players)))
                .await?;
        }
        reply.edit(ctx, |b| b.components(|c| c)).await?;

        Ok(prefs.map(|p| p.unwrap_or_default()))
    }

    pub async fn start(
        &mut self,
        ctx: Context<'_>,
        teams: [TeamPreference; 2],
    ) -> Result<Game, serenity::Error> {
        ctx.say("*Starting lobby...*").await?;

        use TeamPreference::*;
        let mut pair = [self.players[0].id, self.players[1].id];

        if match teams {
            [Either, Either] | [Blue, Blue] | [Red, Red] => {
                ctx.say("Coin toss!").await?;
                rand::random()
            }
            [Red | Either, Blue | Either] => true,
            [Blue | Either, Red | Either] => false,
        } {
            pair.swap(0, 1);
        }

        let game = Game::new(self.id.clone(), pair[0], pair[1]);
        self.status = LobbyStatus::Ongoing;
        Ok(game)
    }
}
