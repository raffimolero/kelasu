use crate::util::input;

use super::types::{Board, Team};
use std::{fmt::Display, str::FromStr};

pub type Pos = u8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    Move { from: Pos, to: Pos },
}

#[derive(Debug)]
pub enum InvalidMove {
    UnknownMove(String),
    MissingParameter(&'static str),
    InvalidParameter(&'static str),
    TileOutOfBounds,
    InvalidPieceMove,
}

impl FromStr for Move {
    type Err = InvalidMove;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens = s.trim().split_whitespace();
        let mut next_token = |on_missing| {
            tokens
                .next()
                .ok_or(InvalidMove::MissingParameter(on_missing))
        };
        let get_pos = |t: &str| {
            const INVALID_POS: InvalidMove =
                InvalidMove::InvalidParameter("Positions must be xy coordinates from 00 to 99.");
            if t.len() != 2 {
                return Err(INVALID_POS);
            }
            t.parse::<Pos>().map_err(|_| INVALID_POS)
        };
        match next_token("Valid moves:\n\tmove xy to xy\n\tmerge xy with xy xy ...")? {
            "move" => {
                let from = next_token("Please specify which position to come from, as an xy coordinate from 00 to 99.")
                    .and_then(get_pos)?;
                if next_token("Syntax: from xy to xy")? != "to" {
                    return Err(InvalidMove::InvalidParameter("Syntax: from xy to xy"));
                }
                let to = next_token(
                    "Please specify which position to go to, as an xy coordinate from 00 to 99.",
                )
                .and_then(get_pos)?;
                Ok(Self::Move { from, to })
            }
            cmd => Err(InvalidMove::UnknownMove(cmd.to_owned())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Game {
    turn: Team,
    remaining_stone_power: u8,
    board: Board,
}

impl Game {
    pub fn new() -> Self {
        Self {
            turn: Team::default(),
            remaining_stone_power: 4,
            board: Board::new(),
        }
    }

    pub fn get_move(&self) -> Result<Move, InvalidMove> {
        let p_move = input("Input a move.").parse::<Move>()?;
        /*
        TODO:
        check bounds - ok
        check if piece exists
        check if move is valid for piece
            check if path is not blocked
            check if not moving towards allies
        */
        todo!()
    }

    pub fn verify_move(&self, p_move: Move) -> Result<(), InvalidMove> {
        todo!()
    }

    pub fn try_move(&mut self, p_move: Move) -> bool {
        todo!()
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?} team's turn.", self.turn)?;
        writeln!(f, "{}", self.board)
    }
}
