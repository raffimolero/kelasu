use kelasu_game::{
    board::{GameState, Move, Pos, VerifiedMove, Winner},
    piece::{Icon, PieceKind, Team},
    Game as BoardGame,
};
use poise::{
    futures_util::StreamExt,
    serenity_prelude::{self as serenity, ButtonStyle, CreateComponents, EmojiId, Message, UserId},
};
use tokio::time::Duration;
use tracing::info;

use crate::{lobby::LobbyId, util::respond_ephemeral, Context};

#[derive(Default, Debug, PartialEq, Eq)]
pub enum TeamPreference {
    Blue,
    #[default]
    Either,
    Red,
}

#[derive(Debug)]
pub struct Game {
    pub lobby: LobbyId,
    pub blue: UserId,
    pub red: UserId,
    pub game: BoardGame,
}

impl Game {
    pub fn new(lobby: LobbyId, blue: UserId, red: UserId) -> Self {
        Self {
            lobby,
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
                for x in 0..11 {
                    if fences[y as usize][x] == 0 {
                        fences[y as usize][x] = 0b110;
                    }
                }
            }

            fences
        };

        fn fence_icon(fence: u8) -> char {
            b" []|()-"[fence as usize] as char
        }

        // this string is only for reference, it will not actually be displayed :P
        // ║═╔╗╚
        let board_repr_len = "\
            ```\n\
            Energy: ########\n\
               ╔-0-1-2-3-4-5-6-7-8-9-╗\n\
             0 ║[B|B(B)B B B B B B[B]║ 0\n\
             1 ║ B B B B B B B B B B ║ 1\n\
             2 ║ s   s  [ | | ]s   s ║ 2\n\
            <3=║-.-.-.-.-.-.-.-.-.-.-║=3>\n\
             4 ║         : :         ║ 4\n\
             5 ║         : :         ║ 5\n\
             6 ║                     ║ 6\n\
             7 ║ s   s         s   s ║ 7\n\
             8 ║ b b b b b b b b b b ║ 8\n\
             9 ║ b b b b b b b b b b ║ 9\n\
               ╚-0-1-2-3-4-5-6-7-8-9-╝\n\
            ```"
        .len();

        let mut out = String::with_capacity(board_repr_len);
        out.push_str("```\nEnergy: ");
        for _ in 0..power {
            out.push('#');
        }
        out.push_str("\n   ╔-0-1-2-3-4-5-6-7-8-9-╗\n");
        for (y, row) in self.game.board.tiles.chunks(10).enumerate() {
            let row_selected = held_digit.map_or(false, |d| d == y as i8);
            let line = if row_selected { b"<=.=>" } else { b"     " }.map(|b| b as char);
            let digit = char::from_digit(y as u32, 10).unwrap();

            out.push(line[0]);
            out.push(digit);
            out.push(line[1]);
            out.push('║');
            for (x, tile) in row.iter().enumerate() {
                out.push(fence_icon(fences[y][x]));
                out.push(if tile.0.is_none() {
                    if [[4, 4], [4, 5], [5, 4], [5, 5]].contains(&[y, x]) {
                        ':'
                    } else {
                        line[2]
                    }
                } else {
                    tile.icon()
                })
            }
            out.push(fence_icon(fences[y][10]));
            out.push('║');
            out.push(line[3]);
            out.push(digit);
            out.push(line[4]);
            out.push('\n');
        }
        out.push_str("   ╚-0-1-2-3-4-5-6-7-8-9-╝\n```");

        out
    }

    async fn select_merge(
        &self,
        ctx: Context<'_>,
        message: &mut Message,
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

        message
            .edit(&ctx.discord().http, |b| {
                b.embed(|e| e.title("What shall be born of this ritual?"))
                    .components(|c| {
                        for row in pieces.chunks(3) {
                            c.create_action_row(|r| {
                                for (kind, emoji) in row {
                                    r.create_button(|b| {
                                        let cost = kind.merge_costs().unwrap();
                                        let disabled = piece_count != cost;
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

        let mut interactions = message.await_component_interactions(ctx.discord()).build();
        let mut interaction;
        let button = loop {
            interaction = interactions.next().await;
            info!("interaction received");
            if let Some(interaction) = &interaction {
                interaction.defer(&ctx.discord().http).await?;
            }
            break match &interaction {
                Some(interaction) if interaction.user.id != player => {
                    respond_ephemeral(ctx, interaction, "It's not your turn.").await?;
                    continue;
                }
                Some(interaction) => {
                    // info!("Deferring");
                    // interaction.defer(&ctx.discord().http).await?;
                    interaction.data.custom_id.as_str()
                }
                None => "cancel",
            };
        };

        message
            .edit(&ctx.discord().http, |m| m.set_embeds(vec![]))
            .await?;

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
        message: &mut Message,
        player: UserId,
        opponent: UserId,
    ) -> Result<bool, serenity::Error> {
        fn ui(c: &mut CreateComponents, disabled: bool) -> &mut CreateComponents {
            c.create_action_row(|r| {
                r.create_button(|b| {
                    b.custom_id("no")
                        .label("No")
                        .emoji('⛔')
                        .style(ButtonStyle::Secondary)
                })
                .create_button(|b| {
                    b.custom_id("resign")
                        .label("Resign")
                        .emoji('⚠')
                        .style(ButtonStyle::Danger)
                        .disabled(disabled)
                })
            })
        }

        let mut disabled = true;
        message
            .edit(&ctx.discord().http, |b| {
                b.embed(|e| e.title("Are you willing to bow down to your opponent?"))
                    .components(|c| ui(c, disabled))
            })
            .await?;

        let mut interactions = message
            .await_component_interactions(ctx.discord())
            .timeout(Duration::from_secs(3))
            .build();
        let mut interaction;
        let button = loop {
            interaction = interactions.next().await;
            info!("interaction received");
            if let Some(interaction) = &interaction {
                interaction.defer(&ctx.discord().http).await?;
            }
            break match &interaction {
                Some(interaction) if interaction.user.id == player => {
                    info!("player move");
                    // info!("Deferring");
                    // interaction.defer(&ctx.discord().http).await?;
                    interaction.data.custom_id.as_str()
                }
                Some(interaction) if interaction.user.id == opponent => {
                    info!("opponent move");
                    respond_ephemeral(ctx, interaction, "It's not your turn.").await?;
                    continue;
                }
                Some(interaction) => {
                    info!("non player move");
                    respond_ephemeral(ctx, interaction, "You're not a player in that lobby.")
                        .await?;
                    continue;
                }
                None if disabled => {
                    disabled = false;
                    interactions = message
                        .await_component_interactions(ctx.discord())
                        .timeout(Duration::from_secs(60 * 5))
                        .build();
                    message
                        .edit(ctx.discord(), |m| m.components(|c| ui(c, disabled)))
                        .await?;
                    continue;
                }
                None => "no",
            };
        };

        let resigned = button == "resign";
        if resigned {
            ctx.say(format!("Very well... <@{player}> has resigned!"))
                .await?;
        }

        message
            .edit(&ctx.discord().http, |m| m.set_embeds(vec![]))
            .await?;
        Ok(resigned)
    }

    async fn offer_draw(
        &self,
        ctx: Context<'_>,
        player: UserId,
        opponent: UserId,
    ) -> Result<VerifiedMove, serenity::Error> {
        let reply = ctx
            .send(|b| {
                b.content(format!("<@{player}> Your opponent is offering a draw."))
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
        let mut message = reply.message().await?.into_owned();

        let mut interactions = message
            .await_component_interactions(ctx.discord())
            .timeout(Duration::from_secs(60 * 5))
            .build();
        let mut interaction;
        let (response, p_move) = loop {
            interaction = interactions.next().await;
            info!("interaction received");
            if let Some(interaction) = &interaction {
                interaction.defer(&ctx.discord().http).await?;
            }
            break match &interaction {
                Some(interaction) => {
                    let from_player = interaction.user.id == player;
                    let from_opponent = interaction.user.id == opponent;
                    info!(from_player, from_opponent);
                    match interaction.data.custom_id.as_str() {
                        "accept" if from_player => ("Accepted.", Move::Draw),
                        "accept" if from_opponent => {
                            respond_ephemeral(ctx, interaction, "It's not your turn.").await?;
                            continue;
                        }
                        "decline" if from_player => ("Declined.", Move::DeclineDraw),
                        "decline" if from_opponent => ("Cancelled.", Move::DeclineDraw),
                        _ => {
                            respond_ephemeral(
                                ctx,
                                interaction,
                                "You are not a player in this game.",
                            )
                            .await?;
                            continue;
                        }
                    }
                }
                None => ("Timed out. Cancelling.", Move::DeclineDraw),
            };
        };
        message
            .edit(&ctx.discord().http, |m| {
                m.content(response).components(|c| c)
            })
            .await?;

        Ok(self.game.verify_move(p_move).unwrap())
    }

    async fn make_move(
        &self,
        ctx: Context<'_>,
        player: UserId,
        opponent: UserId,
        prev_turn: Team,
    ) -> Result<VerifiedMove, serenity::Error> {
        ctx.say(format!(
            "It is {} <@{player}>'s turn. You have 5 minutes to lead.",
            if prev_turn == self.game.turn {
                "still"
            } else {
                "now"
            }
        ))
        .await?;

        let mut held_digit = None;
        let mut positions: Vec<Pos> = Vec::with_capacity(10);

        fn add_components(c: &mut CreateComponents) -> &mut CreateComponents {
            c.create_action_row(|r| {
                (0..5).into_iter().fold(r, |r, i| {
                    r.create_button(|b| b.custom_id(i).label(i).style(ButtonStyle::Secondary))
                })
            })
            .create_action_row(|r| {
                (5..10).fold(r, |r, i| {
                    r.create_button(|b| b.custom_id(i).label(i).style(ButtonStyle::Secondary))
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
                        .emoji(EmojiId(1044186981170683975))
                        .style(ButtonStyle::Success)
                })
            })
        }

        let reply = ctx
            .send(|b| {
                b.content(self.board_repr(self.game.power, &positions, held_digit))
                    .components(|c| add_components(c))
            })
            .await?;

        let mut message = reply.message().await?.into_owned();
        let mut interactions = message
            .await_component_interactions(ctx.discord())
            .timeout(Duration::from_secs(60 * 5))
            .build();
        let mut interaction;
        let p_move = loop {
            interaction = interactions.next().await;
            info!("interaction received");
            if let Some(interaction) = &interaction {
                interaction.defer(&ctx.discord().http).await?;
            }
            let button = match &interaction {
                Some(interaction) if interaction.user.id == player => {
                    info!("player move");
                    interaction.data.custom_id.as_str()
                }
                Some(interaction) if interaction.user.id == opponent => {
                    info!("opponent move");
                    respond_ephemeral(ctx, interaction, "It's not your turn.").await?;
                    continue;
                }
                Some(interaction) => {
                    info!("non player move");
                    respond_ephemeral(ctx, interaction, "You are not a player in this game.")
                        .await?;
                    continue;
                }
                None => {
                    ctx.say("The battle is lost! For you have left your army alone in the darkness for far too long!").await?;
                    "timeout"
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
                "timeout" => MakeMove(Move::Resign),
                "resign" => {
                    let response = self
                        .confirm_resign(ctx, &mut message, player, opponent)
                        .await?;
                    interactions = message
                        .await_component_interactions(ctx.discord())
                        .timeout(Duration::from_secs(60 * 5))
                        .build();
                    if response {
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
                    let response = self
                        .select_merge(ctx, &mut message, player, positions.len())
                        .await?;
                    interactions = message
                        .await_component_interactions(ctx.discord())
                        .timeout(Duration::from_secs(60 * 5))
                        .build();
                    if let Some(kind) = response {
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
            let response = match instruction {
                Noop => None,
                Say(message) => Some(message.to_owned()),
                MakeMove(p_move) => match self.game.verify_move(p_move) {
                    Ok(p_move) => break p_move,
                    Err(e) => Some(format!("Invalid move: {e}")),
                },
                Digit(num) => match held_digit.take() {
                    Some(tens) => {
                        let cursor = Pos(tens * 10 + num);
                        match positions.iter().position(|p| *p == cursor) {
                            Some(idx) => {
                                if idx == positions.len() - 1 {
                                    // selecting the same thing twice pops it
                                    positions.pop();
                                } else {
                                    // selecting once will change the selection
                                    let cursor = positions.remove(idx);
                                    positions.push(cursor);
                                }
                            }
                            None => positions.push(cursor),
                        }
                        None
                    }
                    None => {
                        held_digit = Some(num);
                        None
                    }
                },
                Reset => {
                    reset(&mut held_digit, &mut positions);
                    None
                }
            };
            if let Some(interaction) = interaction {
                if let Some(response) = response {
                    respond_ephemeral(ctx, &interaction, response).await?;
                } else {
                    // info!("deferring...");
                    // interaction.defer(&ctx.discord().http).await?;
                }
            }

            message
                .edit(&ctx.discord().http, |m| {
                    m.content(self.board_repr(self.game.power, &positions, held_digit))
                        .components(|c| add_components(c))
                })
                .await?;
        };

        message
            .edit(&ctx.discord().http, |m| m.components(|c| c))
            .await?;
        Ok(p_move)
    }

    pub async fn start(&mut self, ctx: Context<'_>) -> Result<Winner, serenity::Error> {
        ctx.channel_id()
            .say(
                &ctx.discord().http,
                format!(
                    "**The Battle Begins!**\n\
                    Lobby: {}\n\
                    Blue: <@{}>,\n\
                    Red: <@{}>.\n\
                    May the Great win, and may the Less learn.",
                    self.lobby, self.blue, self.red,
                ),
            )
            .await?;

        let mut prev_turn = !self.game.turn;
        loop {
            let draw_offered = match self.game.state {
                GameState::Ongoing { draw_offered } => draw_offered,
                GameState::Finished(winner) => return Ok(winner),
            };

            let [player, opponent] = match self.game.turn {
                Team::Blue => [self.blue, self.red],
                Team::Red => [self.red, self.blue],
            };

            let p_move = if draw_offered {
                self.offer_draw(ctx, player, opponent).await?
            } else {
                self.make_move(ctx, player, opponent, prev_turn).await?
            };

            prev_turn = self.game.turn;
            self.game.make_move(p_move);
        }
    }
}
