use super::piece::{Icon, MoveKind, Piece, PieceKind, Team, Tile};
use crate::util::input;
use std::{
    fmt::Display,
    ops::{Index, IndexMut},
    str::FromStr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pos(i8);

impl Pos {
    /// returns one of the 8 possible directions. None if knightwise, for example.
    pub fn dir_to(self, rhs: Self) -> Option<([i8; 2], u8)> {
        let x1 = (self.0 % 10) as i8;
        let y1 = (self.0 / 10) as i8;
        let x2 = (rhs.0 % 10) as i8;
        let y2 = (rhs.0 / 10) as i8;
        let dx = x2 - x1;
        let dy = y2 - y1;
        (dx == 0 || dy == 0 || dx.abs() == dy.abs())
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
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Board {
    pub tiles: [Tile; 100],
}

impl Board {
    pub fn new() -> Self {
        // "
        // BBBBBBBBBB
        // BBBBBBBBBB
        // S.S....S.S
        // ..........
        // ....::....
        // ....::....
        // ..........
        // s.s....s.s
        // bbbbbbbbbb
        // bbbbbbbbbb
        // "
        "
        SWWWWWWWWW
        S.........
        S.........
        ..........
        ..........
        ..........
        ..........
        .........s
        .........s
        wwwwwwwwws
        "
        .parse()
        .unwrap()
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
    Move { from: Pos, to: Pos },
    Merge { dest: Pos, blanks: Vec<Pos> },
}

/// just a way to encode trustedness in the type system
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedMove(Move);

#[derive(Debug)]
pub enum InvalidMove {
    Cancelled,
    UnknownMove,
    MissingParameter(&'static str),
    InvalidParameter(&'static str),
    GameOver,
    EmptyTile,
    NotYourPiece,
    FriendlyFire,
    InvalidPieceMove(&'static str),
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
                InvalidMove::InvalidParameter("Positions must be yx coordinates from 00 to 99.");
            if t.len() != 2 {
                return Err(INVALID_POS);
            }
            t.parse::<i8>().map(Pos).map_err(|_| INVALID_POS)
        };
        match next_token("you can type `help`")? {
            "help" => {
                println!("Valid moves:\n\tmove yx to yx\n\tmerge yx with yx yx ...");
                Err(InvalidMove::Cancelled)
            }
            "resign" | "exit" | "quit" => Ok(Self::Resign),
            "move" => {
                let from = next_token("Please specify which position to come from, as an yx coordinate from 00 to 99.")
                    .and_then(get_pos)?;
                if next_token("Syntax: move yx to yx")? != "to" {
                    return Err(InvalidMove::InvalidParameter("Syntax: move yx to yx"));
                }
                let to = next_token(
                    "Please specify which position to go to, as an yx coordinate from 00 to 99.",
                )
                .and_then(get_pos)?;
                Ok(Self::Move { from, to })
            }
            "merge" => todo!(),
            cmd => Err(InvalidMove::UnknownMove),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GameState {
    Ongoing { turn: Team, power: i8 },
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
        let board = Board::new();
        let turn = Team::Blue;
        Self {
            state: GameState::Ongoing {
                turn,
                power: board.stone_count(turn),
            },
            board,
        }
    }

    pub fn is_ongoing(&self) -> bool {
        matches!(self.state, GameState::Ongoing { .. })
    }

    pub fn get_move(&self) -> Result<VerifiedMove, InvalidMove> {
        let p_move = input("Input a move.").parse::<Move>()?;
        self.verify_move(p_move)
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

        if board[to].0.map_or(false, |t| t.team == turn) {
            return Err(InvalidMove::FriendlyFire);
        }

        let ([dx, dy], dist) = from.dir_to(to).ok_or(InvalidMove::InvalidPieceMove(
            "Moves must be in one of 8 directions.",
        ))?;

        let ray_index = Piece::ray_index(dx, dy).unwrap();
        let moves = piece.moves();
        let (move_kind, range) = moves[ray_index];
        if range < dist {
            return Err(InvalidMove::InvalidPieceMove(
                "The destination cannot be reached by that piece.",
            ));
        }

        if move_kind != MoveKind::Recall {
            let mut temp = from;
            for _ in 1..range {
                temp = temp
                    .shift(dx, dy)
                    .expect("from and to are guaranteed to be within bounds.");
                if board[temp].0.is_some() {
                    return Err(InvalidMove::InvalidPieceMove(
                        "There is another piece in the way.",
                    ));
                }
            }
        }

        match move_kind {
            MoveKind::MoveOnly if board[to].0.is_some() => Err(InvalidMove::InvalidPieceMove(
                "There is another piece in the way.",
            )),
            MoveKind::CaptureOnly | MoveKind::Convert if board[to].0.is_none() => {
                Err(InvalidMove::InvalidPieceMove(
                    "That piece must capture something in that direction.",
                ))
            }
            MoveKind::MoveMoveCapture if dist == 1 && board[to].0.is_some() => Err(
                InvalidMove::InvalidPieceMove("Runners cannot capture within a range of 1."),
            ),
            MoveKind::Recall if dist < range => Err(InvalidMove::InvalidPieceMove(
                "Warriors can only return if they are on the opposite row.",
            )),
            _ => Ok(()),
        }
    }

    pub fn verify_move(&self, p_move: Move) -> Result<VerifiedMove, InvalidMove> {
        let GameState::Ongoing { turn, power: _ } = self.state else {
            return Err(InvalidMove::GameOver);
        };
        match p_move {
            Move::Resign => Ok(VerifiedMove(Move::Resign)),
            Move::Move { from, to } => {
                Self::verify_piece_move(&self.board, turn, from, to).map(|_| VerifiedMove(p_move))
            }
            Move::Merge { dest, blanks } => todo!(),
        }
    }

    pub fn make_move(&mut self, p_move: VerifiedMove) {
        let GameState::Ongoing { turn, power } = &mut self.state else {
            unreachable!()
        };
        match p_move.0 {
            Move::Resign => {
                self.state = GameState::Win(turn.flip());
                return;
            }
            Move::Move { from, to } => {
                let is_conversion = self.board[from]
                    .0
                    .map_or(false, |p| p.kind == PieceKind::Diplomat);
                if is_conversion {
                    self.board[to].0.as_mut().unwrap().team = *turn;
                } else {
                    self.board[to] = self.board[from];
                }
                self.board[from].0 = None;
                *power -= 1;
            }
            Move::Merge { dest, blanks } => todo!(),
        }

        // post-move checks
        let victory_by_occupation = [44, 45, 54, 55]
            .map(Pos)
            .into_iter()
            .all(|pos| self.board[pos].0.map_or(false, |p| p.team == *turn));
        if victory_by_occupation {
            self.state = GameState::Win(*turn);
            return;
        }

        let enemy_piece_count = self
            .board
            .tiles
            .iter()
            .filter(|t| t.0.map_or(false, |p| p.team != *turn))
            .count() as i8;
        if enemy_piece_count == 0 {
            self.state = GameState::Win(*turn);
            return;
        }

        let enemy_stone_count = self.board.stone_count(turn.flip());
        if enemy_stone_count == 0 {
            self.state = GameState::Win(*turn);
            return;
        }

        if *power <= 0 {
            *turn = turn.flip();
            *power = enemy_stone_count;
        }
    }
}

#[test]
fn test_diplomat() {
    let mut game = Game::new();
    game.board[Pos(30)] = Tile(Some(Piece {
        team: Team::Blue,
        kind: PieceKind::Diplomat,
    }));
    game.board[Pos(21)] = Tile(Some(Piece {
        team: Team::Red,
        kind: PieceKind::Warrior,
    }));
    println!("{game}");
    game.make_move(
        game.verify_move(Move::Move {
            from: Pos(30),
            to: Pos(21),
        })
        .unwrap(),
    );
    println!("{game}");
}

#[test]
fn test_reverse_move() {
    let mut game = Game {
        state: GameState::Ongoing {
            turn: Team::Red,
            power: 4,
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
    game.verify_move(Move::Move {
        from: Pos(00),
        to: Pos(10),
    })
    .unwrap_err();
    println!("{game}");
}

#[test]
fn test_recall() {
    let mut game = Game {
        state: GameState::Ongoing {
            turn: Team::Red,
            power: 4,
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
