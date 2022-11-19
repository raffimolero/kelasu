use std::time::Duration;

use kelasu_game::{
    board::{GameState, Move, Pos, VerifiedMove, Winner},
    piece::{Icon, Piece, Team},
    Game as BoardGame,
};
use poise::serenity_prelude::{self as serenity, UserId};

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
                b.content(self.game.to_string()).components(|mut c| {
                    c.create_action_row(|r| {
                        r.create_button(|b| {
                            b.custom_id("resign")
                                .label("Resign")
                                .style(serenity::ButtonStyle::Danger)
                        })
                        .create_button(|b| {
                            b.custom_id("draw")
                                .label("Offer Draw")
                                .style(serenity::ButtonStyle::Danger)
                        })
                        .create_button(|b| {
                            b.custom_id("submit")
                                .label("Make Move")
                                .style(serenity::ButtonStyle::Danger)
                        })
                    });
                    for (y, row) in self.game.board.tiles.chunks(10).enumerate() {
                        c.create_action_row(|mut r| {
                            for (x, tile) in row.iter().enumerate() {
                                r.create_button(|b| {
                                    b.custom_id(format!("{y}{x}")).label(tile.icon()).style(
                                        match tile.0 {
                                            Some(Piece {
                                                team: Team::Blue, ..
                                            }) => serenity::ButtonStyle::Primary,
                                            Some(Piece {
                                                team: Team::Red, ..
                                            }) => serenity::ButtonStyle::Danger,
                                            None => serenity::ButtonStyle::Secondary,
                                        },
                                    )
                                });
                            }
                            r
                        });
                    }
                    c
                })
            })
            .await?;

        let message = reply.message().await?;

        let mut positions = vec![];
        loop {
            let interaction = message
                .await_component_interaction(ctx.discord())
                .timeout(Duration::from_secs(60 * 5))
                .author_id(player)
                .await;

            let button_id = match &interaction {
                Some(interaction) => interaction.data.custom_id.as_str(),
                None => {
                    ctx.say("Game Over! You didn't interact in time.").await?;
                    "resign"
                }
            };

            let p_move = match button_id {
                "resign" => Move::Resign,
                "draw" => Move::Draw,
                "submit" => match positions.as_slice() {
                    &[from, to] => Move::Move { from, to },
                    &[_single] => {
                        ctx.say("Where do you want the piece to go?").await?;
                        continue;
                    }
                    pieces => {
                        // check if the number of positions matches a merge
                        // then check if there are ambiguities for merging
                        // if there are, ask the player which one to merge
                        todo!()
                    }
                },
                pos => match pos.parse::<Pos>() {
                    Ok(p) => {
                        // TODO: check if the position is already in the vec. if so, remove it.
                        positions.push(p);
                        continue;
                    } // pushin' 🅿
                    Err(_) => {
                        eprintln!("Unknown button...");
                        continue;
                    }
                },
            };
            match self.game.verify_move(p_move) {
                Ok(p_move) => return Ok(p_move),
                Err(e) => {
                    ctx.say(format!("Invalid move: {e}")).await?;
                }
            }
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
    }
}
