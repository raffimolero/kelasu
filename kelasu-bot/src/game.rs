use std::time::Duration;

use kelasu_game::{
    board::{GameState, Move, Pos, VerifiedMove, Winner},
    piece::{Icon, Piece, Team},
    Game as BoardGame,
};
use poise::{
    futures_util::StreamExt,
    serenity_prelude::{self as serenity, UserId},
};

use crate::Context;

#[derive(Debug, PartialEq, Eq)]
pub enum TeamPreference {
    Blue,
    Either,
    Red,
}

#[derive(Debug)]
pub struct Game {
    pub blue: UserId,
    pub red: UserId,
    pub game: BoardGame,
}

impl Game {
    pub fn new(blue: UserId, red: UserId) -> Self {
        Self {
            blue,
            red,
            game: BoardGame::new(),
        }
    }

    async fn get_move(
        &self,
        ctx: Context<'_>,
        player: UserId,
    ) -> Result<VerifiedMove, serenity::Error> {
        let reply = ctx
            .send(|b| {
                b.content(self.game.to_string()).components(|c| {
                    c.create_action_row(|r| {
                        r.create_button(|b| {
                            b.custom_id("reset")
                                .label("🔁")
                                .style(serenity::ButtonStyle::Secondary)
                        })
                        .create_button(|b| {
                            b.custom_id("up")
                                .label("🔼")
                                .style(serenity::ButtonStyle::Primary)
                        })
                        .create_button(|b| {
                            b.custom_id("submit")
                                .label("✅")
                                .style(serenity::ButtonStyle::Success)
                        })
                    })
                    .create_action_row(|r| {
                        r.create_button(|b| {
                            b.custom_id("left")
                                .label("◀️")
                                .style(serenity::ButtonStyle::Primary)
                        })
                        .create_button(|b| {
                            b.custom_id("select")
                                .label("🅾️")
                                .style(serenity::ButtonStyle::Secondary)
                        })
                        .create_button(|b| {
                            b.custom_id("right")
                                .label("▶️")
                                .style(serenity::ButtonStyle::Primary)
                        })
                    })
                    .create_action_row(|r| {
                        r.create_button(|b| {
                            b.custom_id("draw")
                                .label("🤝")
                                .style(serenity::ButtonStyle::Secondary)
                        })
                        .create_button(|b| {
                            b.custom_id("down")
                                .label("🔽")
                                .style(serenity::ButtonStyle::Primary)
                        })
                        .create_button(|b| {
                            b.custom_id("resign")
                                .label("🏳️")
                                .style(serenity::ButtonStyle::Danger)
                        })
                    })
                })
            })
            .await?;

        let mut message = reply.message().await?.into_owned();

        // let mut positions = vec![];
        let mut interactions = message
            .await_component_interactions(ctx.discord())
            .timeout(Duration::from_secs(60 * 5))
            .author_id(player)
            .build();
        loop {
            message
                .edit(&ctx.discord().http, |m| m.content("new content"))
                .await?;

            let (interaction, rest) = interactions.into_future().await;
            interactions = rest;

            let button_id = match &interaction {
                Some(interaction) => {
                    interaction
                        .create_interaction_response(&ctx.discord().http, |b| {
                            b.interaction_response_data(|r| r.ephemeral(true).content("hi"))
                        })
                        .await?;
                    interaction.data.custom_id.as_str()
                }
                None => {
                    ctx.say("Game Over! You didn't interact in time.").await?;
                    "resign"
                }
            };

            ctx.say(button_id).await?;
            /*
            let p_move = match button_id {
                "resign" => Move::Resign,
                "draw" => Move::Draw,
                "submit" => match positions.as_slice() {
                    [] => {
                        ctx.say("Select a piece.").await?;
                        continue;
                    }
                    &[_single] => {
                        ctx.say("Where do you want the piece to go?").await?;
                        continue;
                    }
                    &[from, to] => Move::Move { from, to },
                    pieces => {
                        // check if the number of positions matches a merge
                        // then check if there are ambiguities for merging
                        // if there are, ask the player which one to merge
                        todo!()
                    }
                },
                other => {
                    eprintln!("Unknown button...");
                    continue;
                }
            };
            match self.game.verify_move(p_move) {
                Ok(p_move) => return Ok(p_move),
                Err(e) => {
                    ctx.say(format!("Invalid move: {e}")).await?;
                    positions = vec![];
                }
            }
            */
        }
    }

    pub async fn start(mut self, ctx: Context<'_>) -> Result<Winner, serenity::Error> {
        loop {
            let draw_offered = match self.game.state {
                GameState::Ongoing { draw_offered } => draw_offered,
                GameState::Finished(winner) => return Ok(winner),
            };

            let player = match self.game.turn {
                Team::Blue => self.blue,
                Team::Red => self.red,
            };
            ctx.say(format!(
                "It's <@{player}>'s turn. You have 5 minutes to move."
            ))
            .await?;

            let p_move = if draw_offered {
                todo!()
            } else {
                self.get_move(ctx, player).await?
            };

            self.game.make_move(p_move);
        }
        // TODO: cleanup
    }
}
