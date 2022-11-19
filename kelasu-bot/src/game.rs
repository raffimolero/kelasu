use kelasu_game::{
    board::{GameState, Move, Pos, VerifiedMove, Winner},
    piece::{Icon, PieceKind, Team},
    Game as BoardGame,
};
use poise::{
    futures_util::StreamExt,
    serenity_prelude::{self as serenity, ButtonStyle, UserId},
};
use tokio::time::{sleep_until, Duration, Instant};

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

    fn board_repr(&self, power: u8, positions: &[Pos], held_digit: Option<i8>) -> String {
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

            for [x, y] in positions.iter().map(xy) {
                fences[y][x + 0] |= 0b_001;
                fences[y][x + 1] |= 0b_010;
            }

            if let Some([x, y]) = positions.last().map(xy) {
                fences[y][x + 0] = 0b_100;
                fences[y][x + 1] = 0b_101;
            }

            if let Some(y) = held_digit {
                fences[y as usize][00] = 0b110;
                fences[y as usize][10] = 0b111;
            }

            fences
        };

        fn fence_icon(fence: u8) -> char {
            b" []|()<>"[fence as usize] as char
        }

        // this string is only for reference, it will not actually be displayed :P
        let board_repr_len = "Power: 8\n\
            ```\n\
               0 1 2 3 4 5 6 7 8 9 \n\
            0 [_|_(_)_ _ _ _ _ _[_]\n\
            1  _ _ _ _ _ _ _ _ _ _ \n\
            2  _ _ _ _[_[_[_]_ _ _ \n\
            3 <_ _ _ _(_)_ _ _ _ _>\n\
            4  _ _ _ _ _ _ _ _ _ _ \n\
            5  _ _ _ _ _ _ _ _ _ _ \n\
            6  _ _ _ _ _ _ _ _ _ _ \n\
            7  _ _ _ _ _ _ _ _ _ _ \n\
            8  _ _ _ _ _ _ _ _ _ _ \n\
            9  _ _ _ _ _ _ _ _ _ _ \n\
            ```"
        .len();

        let mut board_repr = String::with_capacity(board_repr_len);
        board_repr.push_str("Power: ");
        board_repr.push(char::from_digit(power.into(), 10).expect("IT'S OVER 9"));
        board_repr.push_str("\n```\n   0 1 2 3 4 5 6 7 8 9 \n");

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

    async fn select_merge(
        &self,
        ctx: Context<'_>,
        player: UserId,
        piece_count: usize,
    ) -> Result<Option<PieceKind>, serenity::Error> {
        let pieces = [
            (PieceKind::Warrior, '⚔'),
            (PieceKind::Runner, '👟'),
            (PieceKind::Diplomat, '📜'),
            (PieceKind::Champion, '💪'),
            (PieceKind::General, '⭐'),
            (PieceKind::Stone, '💎'),
        ];
        let reply = ctx
            .send(|b| {
                b.content("What piece would you like to merge?")
                    .components(|c| {
                        for row in pieces.chunks(3) {
                            c.create_action_row(|r| {
                                for (kind, emoji) in row {
                                    r.create_button(|b| {
                                        let cost = kind.merge_costs().unwrap();
                                        let disabled = piece_count < cost;
                                        b.custom_id(format!("{kind:?}"))
                                            .label(format!("{kind:?} ({cost})"))
                                            .emoji(*emoji)
                                            .style(if disabled {
                                                ButtonStyle::Secondary
                                            } else {
                                                ButtonStyle::Primary
                                            })
                                            .disabled(disabled)
                                    });
                                }
                                r
                            });
                        }
                        c.create_action_row(|r| {
                            r.create_button(|b| {
                                b.custom_id("cancel")
                                    .label("Cancel")
                                    .emoji('⛔')
                                    .style(ButtonStyle::Danger)
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
            None => "cancel",
        };

        Ok(Some(match button {
            "cancel" => return Ok(None),
            "Warrior" => PieceKind::Warrior,
            "Runner" => PieceKind::Runner,
            "Diplomat" => PieceKind::Diplomat,
            "Champion" => PieceKind::Champion,
            "General" => PieceKind::General,
            "Stone" => PieceKind::Stone,
            _ => panic!("Invalid button ID!"),
        }))
    }

    async fn confirm_resign(
        &self,
        ctx: Context<'_>,
        player: UserId,
    ) -> Result<bool, serenity::Error> {
        let reply = ctx
            .send(|b| {
                b.content("Are you sure you want to resign?\n(Wait 3 seconds)")
                    .components(|c| {
                        c.create_action_row(|r| {
                            r.create_button(|b| {
                                b.custom_id("continue")
                                    .label("Continue")
                                    .emoji('⛔')
                                    .style(ButtonStyle::Secondary)
                            })
                        })
                    })
            })
            .await?;

        let mut message = reply.message().await?;
        let message = message.to_mut();
        let interaction = message
            .await_component_interaction(ctx.discord())
            .author_id(player)
            .timeout(Duration::from_secs(60 * 5));

        // HACK: leverages try_join's behavior on err to return whenever the hell the first block wants to, while the second block waits
        tokio::try_join!(
            async {
                let interaction = interaction.await;
                reply.delete(ctx).await.map_err(Err)?;

                let button = match &interaction {
                    Some(interaction) => interaction.data.custom_id.as_str(),
                    None => "continue",
                };
                return Err::<(), _>(Ok(button == "resign"));
            },
            async {
                sleep_until(Instant::now() + Duration::from_secs(3)).await;
                message
                    .edit(ctx.discord(), |m| {
                        m.components(|c| {
                            c.create_action_row(|r| {
                                r.create_button(|b| {
                                    b.custom_id("continue")
                                        .label("Continue")
                                        .emoji('⛔')
                                        .style(ButtonStyle::Secondary)
                                })
                                .create_button(|b| {
                                    b.custom_id("resign")
                                        .label("Resign")
                                        .emoji('⚠')
                                        .style(ButtonStyle::Danger)
                                })
                            })
                        })
                    })
                    .await
                    .map_err(Err)?;
                Ok(())
            },
        )
        .unwrap_err()
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
                                    .label("Accept")
                                    .emoji('🤝')
                                    .style(ButtonStyle::Secondary)
                            })
                            .create_button(|b| {
                                b.custom_id("decline")
                                    .label("Decline")
                                    .emoji('⛔')
                                    .style(ButtonStyle::Primary)
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

    async fn make_move(
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
                                b.custom_id(i).label(i).style(ButtonStyle::Secondary)
                            })
                        })
                    })
                    .create_action_row(|r| {
                        (5..10).fold(r, |r, i| {
                            r.create_button(|b| {
                                b.custom_id(i).label(i).style(ButtonStyle::Secondary)
                            })
                        })
                    })
                    .create_action_row(|r| {
                        r.create_button(|b| {
                            b.custom_id("resign")
                                .label("Resign")
                                .emoji('⚠')
                                .style(ButtonStyle::Danger)
                        })
                        .create_button(|b| {
                            b.custom_id("draw")
                                .label("Draw")
                                .emoji('🤝')
                                .style(ButtonStyle::Primary)
                        })
                        .create_button(|b| {
                            b.custom_id("reset")
                                .label("Reset")
                                .emoji('🔄')
                                .style(ButtonStyle::Primary)
                        })
                        .create_button(|b| {
                            b.custom_id("move")
                                .label("Move")
                                .emoji('♐')
                                .style(ButtonStyle::Success)
                        })
                        .create_button(|b| {
                            b.custom_id("merge")
                                .label("Merge")
                                .emoji('♻')
                                .style(ButtonStyle::Success)
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
                    m.content(self.board_repr(self.game.power, &positions, held_digit))
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

            fn reset(held_digit: &mut Option<i8>, positions: &mut Vec<Pos>) {
                *held_digit = None;
                positions.clear();
            }

            enum Instruction {
                Noop,
                Say(&'static str),
                MakeMove(Move),
                Digit(i8),
                Reset,
            }
            use Instruction::*;
            let instruction = match button {
                "resign" => {
                    if self.confirm_resign(ctx, player).await? {
                        MakeMove(Move::Resign)
                    } else {
                        Noop
                    }
                }
                "draw" => MakeMove(Move::Draw),
                "reset" => Reset,
                "move" => match positions.as_slice() {
                    [] => Say("Select a piece."),
                    &[_single] => Say("Where do you want the piece to go?"),
                    &[from, to] => MakeMove(Move::Move { from, to }),
                    _ => Say("You selected too many pieces."),
                },
                "merge" => {
                    if let Some(kind) = self.select_merge(ctx, player, positions.len()).await? {
                        MakeMove(Move::Merge {
                            kind,
                            pieces: positions.clone(),
                        })
                    } else {
                        Noop
                    }
                }
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
            match instruction {
                Noop => {}
                Say(message) => {
                    ctx.say(message).await?;
                }
                MakeMove(p_move) => match self.game.verify_move(p_move) {
                    Ok(p_move) => return Ok(p_move),
                    Err(e) => {
                        ctx.say(format!("Invalid move: {e}")).await?;
                        reset(&mut held_digit, &mut positions);
                    }
                },
                Digit(num) => match held_digit.take() {
                    Some(tens) => {
                        let cursor = Pos(tens * 10 + num);
                        match positions.iter().position(|p| *p == cursor) {
                            Some(0) => reset(&mut held_digit, &mut positions),
                            Some(idx) => {
                                positions.remove(idx);
                            }
                            None => positions.push(cursor),
                        }
                    }
                    None => held_digit = Some(num),
                },
                Reset => reset(&mut held_digit, &mut positions),
            }
        }
    }

    pub async fn start(mut self, ctx: Context<'_>) -> Result<Winner, serenity::Error> {
        let mut prev_turn = self.game.turn;
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
                "It's {} <@{player}>'s turn. You have 5 minutes to move.",
                if prev_turn != self.game.turn {
                    "now"
                } else {
                    "still"
                }
            ))
            .await?;

            let p_move = if draw_offered {
                self.offer_draw(ctx, player).await?
            } else {
                self.make_move(ctx, player).await?
            };

            self.game.make_move(p_move);
            prev_turn = self.game.turn;
        }
    }
}
