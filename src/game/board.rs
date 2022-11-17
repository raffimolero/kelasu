use super::piece::{Icon, InvalidPieceMove, MoveKind, Piece, PieceKind, Team, Tile};
use crate::util::{input, verify_polyomino, NonPolyomino};
use std::{
    fmt::Display,
    ops::{Index, IndexMut},
    str::FromStr,
};

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pos(pub i8);

impl Pos {
    /// returns one of the 8 possible directions. None if knightwise, for example.
    pub fn dir_to(self, rhs: Self) -> Option<([i8; 2], u8)> {
        let x1 = (self.0 % 10) as i8;
        let y1 = (self.0 / 10) as i8;
        let x2 = (rhs.0 % 10) as i8;
        let y2 = (rhs.0 / 10) as i8;
        let dx = x2 - x1;
        let dy = y2 - y1;
        ((dx == 0 || dy == 0) ^ (dx.abs() == dy.abs()))
            .then(|| ([dx.signum(), dy.signum()], dx.abs().max(dy.abs()) as u8))
    }

    pub fn shift(mut self, dx: i8, dy: i8) -> Option<Self> {
        let shift_amt = dy * 10 + dx;
        self.0 += shift_amt;
        (0..=99).contains(&self.0).then_some(self)
    }
}

#[test]
fn test_dir_to() {
    assert_eq!(None, Pos(00).dir_to(Pos(21)));
    assert_eq!(None, Pos(21).dir_to(Pos(00)));
    assert_eq!(None, Pos(00).dir_to(Pos(00)));
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Board {
    pub tiles: [Tile; 100],
}

impl Board {
    pub fn new() -> Self {
        "
        BBBBBBBBBB
        BBBBBBBBBB
        S.S....S.S
        ..........
        ....::....
        ....::....
        ..........
        s.s....s.s
        bbbbbbbbbb
        bbbbbbbbbb
        "
        .parse()
        .unwrap()
    }

    pub fn piece_count(&self, team: Team) -> i8 {
        self.tiles
            .iter()
            .filter(|t| {
                t.0.map_or(false, |p| p.team == team && p.kind != PieceKind::Stone)
            })
            .count() as i8
    }

    pub fn stone_count(&self, team: Team) -> i8 {
        let stone = Piece {
            team,
            kind: PieceKind::Stone,
        };
        self.tiles
            .iter()
            .filter(|t| t.0.map_or(false, |p| p == stone))
            .count() as i8
    }
}

impl Index<Pos> for Board {
    type Output = Tile;

    fn index(&self, index: Pos) -> &Self::Output {
        &self.tiles[index.0 as usize]
    }
}

impl IndexMut<Pos> for Board {
    fn index_mut(&mut self, index: Pos) -> &mut Self::Output {
        &mut self.tiles[index.0 as usize]
    }
}

impl FromStr for Board {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some(tiles) = s
            .chars()
            .filter(|c| !c.is_ascii_whitespace())
            .map(|c| c.try_into().ok())
            .collect::<Option<Vec<Tile>>>() else {
            return Err("Invalid tile in string.");
        };

        if tiles.len() < 100 {
            return Err("Not enough tiles.");
        } else if tiles.len() > 100 {
            return Err("Too many tiles.");
        }

        let mut iter = tiles.into_iter();
        let tiles = [(); 100].map(|_| iter.next().unwrap());
        Ok(Self { tiles })
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "   0 1 2 3 4 5 6 7 8 9")?;
        // writeln!(f, "   {}", "-".repeat(21))?;
        for (y, row) in self.tiles.chunks(10).enumerate() {
            write!(f, "{y} ")?;
            for (x, tile) in row.iter().enumerate() {
                let is_victory = [4, 5].contains(&x) && [4, 5].contains(&y) && tile.0.is_none();
                write!(f, "|{}", if is_victory { ':' } else { tile.icon() })?;
            }
            writeln!(f, "|")?;
        }
        // writeln!(f, "   {}", "-".repeat(21))?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Move {
    Resign,
    Move {
        from: Pos,
        to: Pos,
    },
    /// The last position is the destination.
    Merge {
        kind: PieceKind,
        pieces: Vec<Pos>,
    },
}

/// just a way to encode trustedness in the type system
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedMove(Move);

#[derive(Error, Debug)]
pub enum InvalidMove {
    #[error("You cannot move after the game is over.")]
    GameOver,
    #[error("You cannot move an empty tile.")]
    EmptyTile,
    #[error("You cannot move your opponent's pieces.")]
    NotYourPiece,
    #[error("That piece cannot move that way: {0}")]
    InvalidPieceMove(#[from] InvalidPieceMove),
    #[error("Merging that piece requires exactly {0} blanks, including the destination piece.")]
    InvalidMergeCount(usize),
    #[error("You cannot merge into that piece.")]
    InvalidMergeKind,
    #[error(
        "The merging blanks must be next to each other and have no duplicates.\n\
        In this case, {0}."
    )]
    NonPolyominoMerge(#[from] NonPolyomino),
    #[error("You cannot merge pieces in the first two rows of your field.")]
    HomeMerge,
}

#[derive(Error, Debug)]
pub enum InvalidMoveSyntax {
    #[error("The only valid moves are `move` and `merge`.")]
    UnknownMove,
    #[error("Expected another parameter: {0}")]
    MissingParameter(&'static str),
    #[error("That parameter is invalid. {0}")]
    InvalidParameter(&'static str),
}

#[derive(Error, Debug)]
pub enum InvalidMoveCommand {
    #[error(
        "Invalid move syntax: {0}\n\
        Valid moves:\n\
        \tmove <yx> to <yx>\n\
        \tmerge <piece> at <yx> with <yx> <yx> ..."
    )]
    InvalidSyntax(#[from] InvalidMoveSyntax),
    #[error("That move is illegal: {0}")]
    InvalidMove(#[from] InvalidMove),
}

impl FromStr for Move {
    type Err = InvalidMoveCommand;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let clean_str = s.trim().to_ascii_lowercase();
        let mut tokens = clean_str.split_whitespace();
        let mut next_token = |on_missing| {
            tokens
                .next()
                .ok_or(InvalidMoveSyntax::MissingParameter(on_missing))
        };

        let get_pos = |t: &str| {
            const INVALID_POS: InvalidMoveSyntax = InvalidMoveSyntax::InvalidParameter(
                "Positions must be <yx> coordinates from 00 to 99.",
            );
            if t.len() != 2 {
                return Err(INVALID_POS);
            }
            t.parse::<i8>().map(Pos).map_err(|_| INVALID_POS)
        };

        match next_token("What kind of move did you want to make?")? {
            "resign" | "exit" | "quit" => Ok(Self::Resign),
            "move" => {
                let from = next_token("From where?").and_then(get_pos)?;
                if next_token("To where?")? != "to" {
                    Err(InvalidMoveSyntax::InvalidParameter("missing 'to'"))?;
                }
                let to = next_token("Where to?").and_then(get_pos)?;
                Ok(Self::Move { from, to })
            }
            "merge" => {
                let (kind, cost) = next_token("What do you want to merge into?")?
                    .parse::<PieceKind>()
                    .map_err(|_| {
                        InvalidMoveSyntax::InvalidParameter(
                            "Specify what kind of piece you want to merge into.",
                        )
                    })
                    .map_err(InvalidMoveCommand::from)
                    .and_then(|k| {
                        k.merge_costs()
                            .map(|v| (k, v))
                            .ok_or(InvalidMove::InvalidMergeKind.into())
                    })?;

                if next_token("At where?")? != "at" {
                    Err(InvalidMoveSyntax::MissingParameter("missing 'at'"))?;
                }

                let dest = next_token("At where?").and_then(get_pos)?;

                if next_token("With which other pieces?")? != "with" {
                    Err(InvalidMoveSyntax::MissingParameter("missing 'with'"))?;
                }

                let mut pieces = (tokens)
                    .take(cost)
                    .map(get_pos)
                    .collect::<Result<Vec<Pos>, InvalidMoveSyntax>>()?;

                if pieces.len() != cost - 1 {
                    return Err(InvalidMoveCommand::InvalidMove(
                        InvalidMove::InvalidMergeCount(cost),
                    ));
                }

                pieces.push(dest);
                Ok(Self::Merge { kind, pieces })
            }
            _ => Err(InvalidMoveSyntax::UnknownMove)?,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GameState {
    Ongoing {
        turn: Team,
        power: i8,
        stagnation: u8,
    },
    Win(Team),
    Draw,
}

impl Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameState::Ongoing { turn, power, .. } => {
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
        let board = Board::new();
        let turn = Team::Blue;
        Self {
            state: GameState::Ongoing {
                turn,
                power: board.stone_count(turn),
                stagnation: 0,
            },
            board,
        }
    }

    pub fn is_ongoing(&self) -> bool {
        matches!(self.state, GameState::Ongoing { .. })
    }

    pub fn get_move(&self) -> Result<VerifiedMove, InvalidMoveCommand> {
        let p_move = input("Input a move.").parse::<Move>()?;
        self.verify_move(p_move)
            .map_err(InvalidMoveCommand::InvalidMove)
    }

    pub fn verify_piece_move(
        board: &Board,
        turn: Team,
        from: Pos,
        to: Pos,
    ) -> Result<(), InvalidMove> {
        // we don't have to check for power because it should immediately switch turns then

        let Tile(Some(piece)) = board[from] else {
            return Err(InvalidMove::EmptyTile);
        };

        if piece.team != turn {
            return Err(InvalidMove::NotYourPiece);
        }

        let ([dx, dy], dist) = from.dir_to(to).ok_or(InvalidPieceMove::NonCompassMove)?;

        let ray_index = Piece::ray_index(dx, dy).unwrap();
        let moves = piece.moves();
        let (move_kind, range) = moves[ray_index];
        if range < dist {
            Err(InvalidPieceMove::TooFar)?;
        }

        if move_kind != MoveKind::Recall {
            let mut temp = from;
            for _ in 1..range {
                temp = temp
                    .shift(dx, dy)
                    .expect("from and to are guaranteed to be within bounds.");
                if board[temp].0.is_some() {
                    Err(InvalidPieceMove::Blocked)?;
                }
            }
        }

        if board[to].0.map_or(false, |t| t.team == turn) {
            Err(InvalidPieceMove::FriendlyFire)?;
        }

        match move_kind {
            MoveKind::MoveOnly if board[to].0.is_some() => Err(InvalidPieceMove::Blocked),
            MoveKind::CaptureOnly | MoveKind::Convert if board[to].0.is_none() => {
                Err(InvalidPieceMove::MustCapture)
            }
            MoveKind::MoveMoveCapture if dist == 1 && board[to].0.is_some() => {
                Err(InvalidPieceMove::RunnerNoMelee)
            }
            MoveKind::Recall if dist < range => Err(InvalidPieceMove::CannotRecallHere),
            _ => Ok(()),
        }?;
        Ok(())
    }

    /// the number of pieces required for the merge kind is checked during parsing
    ///
    /// juuust in case people specify 10,000 different pieces
    pub fn verify_merge(board: &Board, turn: Team, pieces: &mut [Pos]) -> Result<(), InvalidMove> {
        let home_rows = match turn {
            Team::Blue => 00..20,
            Team::Red => 90..100,
        };

        for p in pieces.iter() {
            let Some(piece) = board[*p].0 else {
                return Err(InvalidMove::EmptyTile);
            };
            if piece.team != turn {
                return Err(InvalidMove::NotYourPiece);
            }
            if piece.kind != PieceKind::Blank {
                Err(InvalidPieceMove::NonBlankMerge)?;
            }
            if home_rows.contains(&p.0) {
                return Err(InvalidMove::HomeMerge);
            }
        }

        verify_polyomino(pieces)?;

        Ok(())
    }

    pub fn verify_move(&self, mut p_move: Move) -> Result<VerifiedMove, InvalidMove> {
        let GameState::Ongoing { turn, .. } = self.state else {
            return Err(InvalidMove::GameOver);
        };
        match &mut p_move {
            Move::Resign => Ok(()),
            Move::Move { from, to } => Self::verify_piece_move(&self.board, turn, *from, *to),
            Move::Merge { kind: _, pieces } => Self::verify_merge(&self.board, turn, pieces),
        }
        .map(|_| VerifiedMove(p_move))
    }

    pub fn make_move(&mut self, p_move: VerifiedMove) {
        let GameState::Ongoing { turn: us, power, stagnation } = &mut self.state else {
            panic!("make_move must only be called while the game is ongoing.");
        };
        let them = !*us;

        match p_move.0 {
            Move::Resign => {
                self.state = GameState::Win(them);
                return;
            }
            Move::Move { from, to } => {
                *power -= 1;
                let kind = self.board[from].0.unwrap().kind;
                if kind == PieceKind::Blank {
                    *stagnation = 0;
                }
                // check if a diplomat made a diagonal move, i.e. when neither x nor y are 0
                if kind == PieceKind::Diplomat && !from.dir_to(to).unwrap().0.contains(&0) {
                    // convert the piece
                    self.board[to].0.as_mut().unwrap().team = *us;
                } else {
                    self.board[to] = self.board[from];
                }
                self.board[from].0 = None;
            }
            Move::Merge { kind, mut pieces } => {
                *stagnation = 0;
                *power -= pieces.len() as i8;
                let dest = pieces.pop().unwrap();
                for pos in pieces {
                    self.board[pos].0 = None;
                }
                // transform the piece
                self.board[dest].0.as_mut().unwrap().kind = kind;
            }
        }

        // post-move checks
        let victory_by_occupation = [44, 45, 54, 55]
            .map(Pos)
            .into_iter()
            .all(|pos| self.board[pos].0.map_or(false, |p| p.team == *us));
        if victory_by_occupation {
            self.state = GameState::Win(*us);
            return;
        }

        let enemy_piece_count = self.board.piece_count(them);
        let enemy_stone_count = self.board.stone_count(them);
        if enemy_piece_count == 0 || enemy_stone_count == 0 {
            self.state = GameState::Win(*us);
            return;
        }

        if *power <= 0 {
            *us = them;
            *power = enemy_stone_count;
            if *us == Team::Blue {
                *stagnation += 1;
                if *stagnation > 64 {
                    self.state = GameState::Draw;
                }
            }
        }
    }
}

#[test]
fn test_diplomat() {
    let mut game = Game {
        state: GameState::Ongoing {
            turn: Team::Blue,
            power: 4,
            stagnation: 0,
        },
        board: "
            sS........
            ..........
            .w........
            D.........
            ..........
            ..........
            ..........
            ....w..D..
            ..........
            ..........
        "
        .parse()
        .unwrap(),
    };
    game.make_move(
        game.verify_move(Move::Move {
            from: Pos(30),
            to: Pos(21),
        })
        .unwrap(),
    );
    assert_eq!(
        game.board,
        "
        sS........
        ..........
        .W........
        ..........
        ..........
        ..........
        ..........
        ....w..D..
        ..........
        ..........
        "
        .parse()
        .unwrap()
    );
    game.verify_move(Move::Move {
        from: Pos(77),
        to: Pos(74),
    })
    .unwrap_err();
    game.make_move(
        game.verify_move(Move::Move {
            from: Pos(77),
            to: Pos(47),
        })
        .unwrap(),
    );
    assert_eq!(
        game.board,
        "
        sS........
        ..........
        .W........
        ..........
        .......D..
        ..........
        ..........
        ....w.....
        ..........
        ..........
        "
        .parse()
        .unwrap()
    );
}

#[test]
fn test_reverse_move() {
    let game = Game {
        state: GameState::Ongoing {
            turn: Team::Red,
            power: 4,
            stagnation: 0,
        },
        board: "
            w.........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
        "
        .parse()
        .unwrap(),
    };
    game.verify_move(Move::Move {
        from: Pos(00),
        to: Pos(10),
    })
    .unwrap_err();
}

#[test]
fn test_recall() {
    let mut game = Game {
        state: GameState::Ongoing {
            turn: Team::Red,
            power: 4,
            stagnation: 0,
        },
        board: "
            w.........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
        "
        .parse()
        .unwrap(),
    };
    println!("{game}");
    game.make_move(
        game.verify_move(Move::Move {
            from: Pos(00),
            to: Pos(90),
        })
        .unwrap(),
    );
    println!("{game}");
}

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}{}", self.state, self.board)
    }
}
