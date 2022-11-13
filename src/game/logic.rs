use super::types::{Board, Team};
use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    x: usize,
    y: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    Move { from: Pos, to: Pos },
}

#[derive(Debug)]
pub enum InvalidMove {
    EmptyMove,
    UnknownMove(String),
    TileOutOfBounds,
    InvalidPieceMove,
}

impl FromStr for Move {
    type Err = InvalidMove;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens = s.trim().split_whitespace();
        match tokens.next() {
            Some(cmd) => Err(InvalidMove::UnknownMove(cmd.to_owned())),
            None => Err(InvalidMove::EmptyMove),
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
