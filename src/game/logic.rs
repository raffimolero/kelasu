use super::types::{Board, Piece, Pos, Team, Tile};
use crate::util::input;
use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    Move { from: Pos, to: Pos },
}

#[derive(Debug)]
pub enum InvalidMove {
    UnknownMove(String),
    MissingParameter(&'static str),
    InvalidParameter(&'static str),
    GameOver,
    EmptyTile,
    NotYourPiece,
    FriendlyFire,
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
pub enum GameState {
    Ongoing { turn: Team, power: u8 },
    Win(Team),
    Draw,
}

impl Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameState::Ongoing { turn, power } => {
                writeln!(f, "{turn:?}'s turn.")?;
                writeln!(f, "Remaining Stone Power: {power}.")
            }
            GameState::Win(team) => writeln!(f, "Winner: {team:?}."),
            GameState::Draw => writeln!(f, "Draw."),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Game {
    state: GameState,
    board: Board,
}

impl Game {
    pub fn new() -> Self {
        Self {
            state: GameState::Ongoing {
                turn: Team::Blue,
                power: 4,
            },
            board: Board::new(),
        }
    }

    pub fn get_move(&self) -> Result<Move, InvalidMove> {
        /*
        TODO:
        check bounds - ok
        check if piece exists - ok
        check if move is valid for piece
            check if path is not blocked
            check if not moving towards allies
        */
        let GameState::Ongoing { turn, power } = self.state else {
            return Err(InvalidMove::GameOver);
        };
        let p_move = input("Input a move.").parse::<Move>()?;
        match p_move {
            Move::Move { from, to } => {
                let Tile(Some(from)) = self.board[from] else {
                    return Err(InvalidMove::EmptyTile);
                };
                if from.team != turn {
                    return Err(InvalidMove::NotYourPiece);
                }
                if self.board[to].0.map_or(false, |t| t.team == turn) {
                    return Err(InvalidMove::FriendlyFire);
                }
            }
        }
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
        writeln!(f, "{}", self.state)?;
        writeln!(f, "{}", self.board)
    }
}
