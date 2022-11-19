use std::time::Duration;

use kelasu_game::{
    board::{GameState, Move, Pos, VerifiedMove, Winner},
    piece::{Icon, Team},
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

    fn board_repr(&self, positions: &[Pos]) -> String {
        /*
        000: None
        001: Left
        010: Right
        011: Both
        100: Selected Left
        101: Selected Right
        110: Cursor Left
        111: Cursor Right
        */

        fn xy(&Pos(p): &Pos) -> [usize; 2] {
            [p as usize % 10, p as usize / 10]
        }

        let fences = {
            let mut fences = [[0b_000_u8; 11]; 10];
            for [x, y] in positions.iter().skip(1).map(xy) {
                fences[y][x + 0] |= 0b_001;
                fences[y][x + 1] |= 0b_010;
            }

            if let Some([x, y]) = positions.first().map(xy) {
                fences[y][x + 0] = 0b_100;
                fences[y][x + 1] = 0b_101;
            }

            fences
        };

        fn fence_icon(fence: u8) -> char {
            b" []|<>"[fence as usize] as char
        }

        // this string is only for reference, it will not actually be displayed :P
        let board_repr_len = "```\n\
            0 1 2 3 4 5 6 7 8 9 \n\
            0 [_[_(_)_ _ _ _ _ _[_]\n\
            1  _ _ _ _ _ _ _ _ _ _ \n\
            2  _ _ _ _[_[_[_]_ _ _ \n\
            3  _ _ _<_>_)_ _ _ _ _ \n\
            4  _ _ _ _ _ _ _ _ _ _ \n\
            5  _ _ _ _ _ _ _ _ _ _ \n\
            6  _ _ _ _ _ _ _ _ _ _ \n\
            7  _ _ _ _ _ _ _ _ _ _ \n\
            8  _ _ _ _ _ _ _ _ _ _ \n\
            9  _ _ _ _ _ _ _ _ _ _ \n\
            ```"
        .len();

        let mut board_repr = String::with_capacity(board_repr_len);
        board_repr.push_str("```\n   0 1 2 3 4 5 6 7 8 9 \n");

        for (y, row) in self.game.board.tiles.chunks(10).enumerate() {
            board_repr.push(char::from_digit(y as u32, 10).unwrap());
            board_repr.push(' ');
            for (x, tile) in row.iter().enumerate() {
                board_repr.push(fence_icon(fences[y][x]));
                board_repr.push(tile.icon());
            }
            board_repr.push(fence_icon(fences[y][10]));
            board_repr.push('\n');
        }
        board_repr.push_str("```");

        board_repr
    }

    async fn get_move_pos(
        &self,
        ctx: Context<'_>,
        player: UserId,
    ) -> Result<VerifiedMove, serenity::Error> {
        let reply = ctx
            .send(|b| {
                b.content("loading...").components(|c| {
                    c.create_action_row(|r| {
                        (0..5).into_iter().fold(r, |r, i| {
                            r.create_button(|b| {
                                b.custom_id(i)
                                    .label(i)
                                    .style(serenity::ButtonStyle::Secondary)
                            })
                        })
                    })
                    .create_action_row(|r| {
                        (5..10).fold(r, |r, i| {
                            r.create_button(|b| {
                                b.custom_id(i)
                                    .label(i)
                                    .style(serenity::ButtonStyle::Secondary)
                            })
                        })
                    })
                    .create_action_row(|r| {
                        r.create_button(|b| {
                            b.custom_id("reset")
                                .label("🔁")
                                .style(serenity::ButtonStyle::Secondary)
                        })
                        .create_button(|b| {
                            b.custom_id("resign")
                                .label("🏳️")
                                .style(serenity::ButtonStyle::Danger)
                        })
                        .create_button(|b| {
                            b.custom_id("draw")
                                .label("🤝")
                                .style(serenity::ButtonStyle::Secondary)
                        })
                        .create_button(|b| {
                            b.custom_id("submit")
                                .label("✅")
                                .style(serenity::ButtonStyle::Success)
                        })
                    })
                })
            })
            .await?;

        let mut message = reply.message().await?.into_owned();

        let mut held_digit = None;
        let mut positions: Vec<Pos> = Vec::with_capacity(10);
        let mut interactions = message
            .await_component_interactions(ctx.discord())
            .timeout(Duration::from_secs(60 * 5))
            .author_id(player)
            .build();

        loop {
            message
                .edit(&ctx.discord().http, |m| {
                    m.content(self.board_repr(&positions))
                })
                .await?;

            let (interaction, rest) = interactions.into_future().await;
            interactions = rest;

            let button = match &interaction {
                Some(interaction) => {
                    // HACK: create and delete a response so discord knows something happened
                    interaction
                        .create_interaction_response(&ctx.discord().http, |b| {
                            b.interaction_response_data(|r| r.ephemeral(true).content("processed."))
                        })
                        .await?;
                    interaction
                        .delete_original_interaction_response(&ctx.discord().http)
                        .await?;
                    interaction.data.custom_id.as_str()
                }
                None => {
                    ctx.say("Game Over! You didn't interact in time.").await?;
                    "resign"
                }
            };

            enum Input {
                Say(&'static str),
                InstantMove(Move),
                Digit(i8),
                Reset,
            }
            use Input::*;
            let input = match button {
                "resign" => InstantMove(Move::Resign),
                "draw" => InstantMove(Move::Draw),
                "reset" => Reset,
                "submit" => match positions.as_slice() {
                    [] => Say("Select a piece."),
                    &[_single] => Say("Where do you want the piece to go?"),
                    &[from, to] => InstantMove(Move::Move { from, to }),
                    pieces => {
                        // check if the number of positions matches a merge
                        // then check if there are ambiguities for merging
                        // if there are, ask the player which one to merge
                        Say("Piece merging is not implemented yet '~'")
                    }
                },
                "0" => Digit(0),
                "1" => Digit(1),
                "2" => Digit(2),
                "3" => Digit(3),
                "4" => Digit(4),
                "5" => Digit(5),
                "6" => Digit(6),
                "7" => Digit(7),
                "8" => Digit(8),
                "9" => Digit(9),
                _ => Say("Unknown button..."),
            };
            match input {
                Say(message) => {
                    ctx.say(message).await?;
                }
                InstantMove(p_move) => match self.game.verify_move(p_move) {
                    Ok(p_move) => return Ok(p_move),
                    Err(e) => {
                        ctx.say(format!("Invalid move: {e}")).await?;
                        positions = vec![];
                    }
                },
                Digit(num) => match held_digit.take() {
                    Some(tens) => {
                        let cursor = Pos(tens * 10 + num);
                        if let Some(idx) = positions.iter().position(|p| *p == cursor) {
                            positions.swap_remove(idx);
                        } else {
                            positions.push(cursor);
                        }
                    }
                    None => held_digit = Some(num),
                },
                Reset => positions.clear(),
            }
        }
    }

    async fn offer_draw(
        &self,
        ctx: Context<'_>,
        player: UserId,
    ) -> Result<VerifiedMove, serenity::Error> {
        let reply = ctx
            .send(|b| {
                b.content("Your opponent is offering a draw.")
                    .components(|c| {
                        c.create_action_row(|r| {
                            r.create_button(|b| {
                                b.custom_id("accept")
                                    .label("🤝")
                                    .style(serenity::ButtonStyle::Secondary)
                            })
                            .create_button(|b| {
                                b.custom_id("decline")
                                    .label("❎")
                                    .style(serenity::ButtonStyle::Primary)
                            })
                        })
                    })
            })
            .await?;

        let interaction = reply
            .message()
            .await?
            .await_component_interaction(ctx.discord())
            .author_id(player)
            .timeout(Duration::from_secs(60 * 5))
            .await;

        reply.delete(ctx).await?;

        let button = match &interaction {
            Some(interaction) => interaction.data.custom_id.as_str(),
            None => "decline",
        };

        let p_move = match button {
            "accept" => Move::Draw,
            "decline" => Move::DeclineDraw,
            _ => panic!("Invalid button ID!"),
        };

        Ok(self.game.verify_move(p_move).unwrap())
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
                self.offer_draw(ctx, player).await?
            } else {
                self.get_move_pos(ctx, player).await?
            };

            self.game.make_move(p_move);
        }
        // TODO: cleanup
    }
}
