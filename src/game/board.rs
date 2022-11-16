use super::piece::{Icon, MoveKind, Piece, PieceKind, Team, Tile};
use crate::util::{input, is_polyomino};
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
        ((dx & dy == 0) ^ (dx.abs() == dy.abs()))
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
    Merge { pieces: Vec<Pos> },
}

impl Move {
    pub const MERGE_KINDS: [PieceKind; 6] = [
        PieceKind::Warrior,
        PieceKind::Runner,
        PieceKind::Diplomat,
        PieceKind::Champion,
        PieceKind::General,
        PieceKind::Stone, // it must be done
    ];
    pub const MERGE_COSTS: [usize; 6] = [2, 4, 4, 5, 10, 21];
}

/// just a way to encode trustedness in the type system
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedMove(Move);

#[derive(Debug)]
pub enum IllegalMove {
    GameOver,
    EmptyTile,
    NotYourPiece,
    FriendlyFire,
    InvalidPieceMove(&'static str),
    // TODO: thiserror, expected exactly {0} other blanks after the first.
    InvalidMergeCount,
    InvalidMergeKind,
    DisconnectedMerges,
    HomeMerge,
}

#[derive(Debug)]
pub enum InvalidMoveCommand {
    Help,
    UnknownMove,
    Illegal(IllegalMove),
    MissingParameter(&'static str),
    InvalidParameter(&'static str),
    // TODO: replace with thiserror
    InvalidPosition(&'static str),
}

impl FromStr for Move {
    type Err = InvalidMoveCommand;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let clean_str = s.trim().to_ascii_lowercase();
        let mut tokens = clean_str.split_whitespace();
        let mut next_token = |on_missing| {
            tokens
                .next()
                .ok_or(InvalidMoveCommand::MissingParameter(on_missing))
        };

        let get_pos = |t: &str| {
            const INVALID_POS: InvalidMoveCommand = InvalidMoveCommand::InvalidPosition(
                "Positions must be <yx> coordinates from 00 to 99.",
            );
            if t.len() != 2 {
                return Err(INVALID_POS);
            }
            t.parse::<i8>().map(Pos).map_err(|_| INVALID_POS)
        };

        match next_token("you can type `help`")? {
            "help" => Err(InvalidMoveCommand::Help),
            "resign" | "exit" | "quit" => Ok(Self::Resign),
            "move" => {
                let from = next_token("From where?").and_then(get_pos)?;
                if next_token("To where?")? != "to" {
                    return Err(InvalidMoveCommand::InvalidParameter(
                        "Syntax: move <yx> to <yx>",
                    ));
                }
                let to = next_token("Where to?").and_then(get_pos)?;
                Ok(Self::Move { from, to })
            }
            "merge" => {
                let cost = next_token("What do you want to merge into?")?
                    .parse::<PieceKind>()
                    .map_err(|_| {
                        InvalidMoveCommand::InvalidParameter(
                            "Specify what kind of piece you want to merge into.",
                        )
                    })
                    .and_then(|k| {
                        k.merge_costs()
                            .ok_or(InvalidMoveCommand::Illegal(IllegalMove::InvalidMergeKind))
                    })?;

                if next_token("At where?")? != "at" {
                    return Err(InvalidMoveCommand::InvalidParameter(
                        "Syntax: merge <piece> at <yx> with <yx> <yx> ...",
                    ));
                }

                let dest = next_token("At where?").and_then(get_pos)?;

                if next_token("With which other pieces?")? != "with" {
                    return Err(InvalidMoveCommand::InvalidParameter(
                        "Syntax: merge <piece> at <yx> with <yx> <yx> ...",
                    ));
                }

                let blank_ct = cost - 1;
                let mut pieces = (tokens)
                    .take(blank_ct + 1)
                    .map(get_pos)
                    .collect::<Result<Vec<Pos>, InvalidMoveCommand>>()?;

                if pieces.len() != blank_ct {
                    return Err(InvalidMoveCommand::Illegal(IllegalMove::InvalidMergeCount));
                }

                pieces.push(dest);
                Ok(Self::Merge { pieces })
            }
            _ => Err(InvalidMoveCommand::UnknownMove),
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

    pub fn get_move(&self) -> Result<VerifiedMove, InvalidMoveCommand> {
        let p_move = input("Input a move.").parse::<Move>()?;
        self.verify_move(p_move)
            .map_err(InvalidMoveCommand::Illegal)
    }

    pub fn verify_piece_move(
        board: &Board,
        turn: Team,
        from: Pos,
        to: Pos,
    ) -> Result<(), IllegalMove> {
        // we don't have to check for power because it should immediately switch turns then

        let Tile(Some(piece)) = board[from] else {
            return Err(IllegalMove::EmptyTile);
        };

        if piece.team != turn {
            return Err(IllegalMove::NotYourPiece);
        }

        let ([dx, dy], dist) = from.dir_to(to).ok_or(IllegalMove::InvalidPieceMove(
            "Moves must be in one of 8 directions.",
        ))?;

        let ray_index = Piece::ray_index(dx, dy).unwrap();
        let moves = piece.moves();
        let (move_kind, range) = moves[ray_index];
        if range < dist {
            return Err(IllegalMove::InvalidPieceMove(
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
                    return Err(IllegalMove::InvalidPieceMove(
                        "There is another piece in the way.",
                    ));
                }
            }
        }

        if board[to].0.map_or(false, |t| t.team == turn) {
            return Err(IllegalMove::FriendlyFire);
        }

        match move_kind {
            MoveKind::MoveOnly if board[to].0.is_some() => Err(IllegalMove::InvalidPieceMove(
                "There is another piece in the way.",
            )),
            MoveKind::CaptureOnly | MoveKind::Convert if board[to].0.is_none() => {
                Err(IllegalMove::InvalidPieceMove(
                    "That piece must capture something in that direction.",
                ))
            }
            MoveKind::MoveMoveCapture if dist == 1 && board[to].0.is_some() => Err(
                IllegalMove::InvalidPieceMove("Runners cannot capture within a range of 1."),
            ),
            MoveKind::Recall if dist < range => Err(IllegalMove::InvalidPieceMove(
                "Warriors can only return if they are on the opposite row.",
            )),
            _ => Ok(()),
        }
    }

    pub fn verify_merge(board: &Board, turn: Team, pieces: &[Pos]) -> Result<(), IllegalMove> {
        let home_rows = match turn {
            Team::Blue => 00..20,
            Team::Red => 90..100,
        };
        if pieces.iter().any(|p| home_rows.contains(&p.0)) {
            return Err(IllegalMove::HomeMerge);
        }

        if !is_polyomino(pieces) {
            return Err(IllegalMove::DisconnectedMerges);
        }

        Ok(())
    }

    pub fn verify_move(&self, p_move: Move) -> Result<VerifiedMove, IllegalMove> {
        let GameState::Ongoing { turn, power: _ } = self.state else {
            return Err(IllegalMove::GameOver);
        };
        match &p_move {
            Move::Resign => Ok(()),
            Move::Move { from, to } => Self::verify_piece_move(&self.board, turn, *from, *to),
            Move::Merge { pieces } => Self::verify_merge(&self.board, turn, &pieces),
        }
        .map(|_| VerifiedMove(p_move))
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
            Move::Merge { pieces } => todo!(),
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
