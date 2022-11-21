use std::{ops::Not, str::FromStr};

use thiserror::Error;

/// a single-character "icon" that an object can have
pub trait Icon {
    fn icon(&self) -> char;
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Team {
    #[default]
    Blue,
    Red,
}

impl Not for Team {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Team::Blue => Team::Red,
            Team::Red => Team::Blue,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveKind {
    MoveOnly,
    CaptureOnly,
    MoveCapture,
    MoveMoveCapture,
    Recall,
    Convert,
}

#[derive(Error, Debug)]
pub enum InvalidPieceMove {
    #[error("Moves must be either orthogonal or diagonal.")]
    NonCompassMove,
    #[error("The piece can't move that far in that direction.")]
    TooFar,
    #[error("There is another piece in the way.")]
    Blocked,
    #[error("That piece must capture something in that direction.")]
    MustCapture,
    #[error("You cannot capture or move into your own pieces.")]
    FriendlyFire,
    #[error("Runners cannot capture within a range of 1.")]
    NoMelee,
    #[error("Warriors can only return if they are on the opposite row.")]
    CannotRecallHere,
    #[error("Only Blanks can merge together.")]
    NonBlankMerge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PieceKind {
    Blank,
    Warrior,
    Runner,
    Diplomat,
    Champion,
    General,
    Stone,
}

impl PieceKind {
    /// order: forward, fore-side, side, back-side, back.
    ///
    /// format: (kind, maxrange)
    pub fn moves(self) -> [(MoveKind, u8); 5] {
        use MoveKind::*;
        use PieceKind::*;
        match self {
            Blank => [
                (MoveOnly, 1),
                (MoveOnly, 0),
                (MoveOnly, 1),
                (MoveOnly, 0),
                (MoveOnly, 0),
            ],
            Warrior => [
                (MoveCapture, 1),
                (CaptureOnly, 1),
                (MoveCapture, 1),
                (MoveOnly, 0),
                (Recall, 9),
            ],
            Runner => [
                (MoveOnly, 0),
                (MoveMoveCapture, 10),
                (MoveOnly, 0),
                (MoveMoveCapture, 10),
                (MoveOnly, 0),
            ],
            Diplomat => [
                (MoveOnly, 3),
                (Convert, 1),
                (MoveOnly, 3),
                (Convert, 1),
                (MoveOnly, 3),
            ],
            Champion => [
                (MoveCapture, 10),
                (MoveCapture, 1),
                (MoveCapture, 3),
                (MoveOnly, 0),
                (MoveOnly, 10),
            ],
            General => [(MoveCapture, 10); 5],
            Stone => [(MoveOnly, 0); 5],
        }
    }

    pub fn merge_costs(self) -> Option<usize> {
        Some(match self {
            PieceKind::Warrior => 2,
            PieceKind::Runner => 4,
            PieceKind::Diplomat => 4,
            PieceKind::Champion => 5,
            PieceKind::General => 10,
            PieceKind::Stone => 21,
            _ => return None,
        })
    }
}

#[derive(Error, Debug)]
#[error("I don't recognize that piece. Check for spelling issues.")]
pub struct UnknownPiece;

impl FromStr for PieceKind {
    type Err = UnknownPiece;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_ascii_lowercase().as_str() {
            "blank" => Self::Blank,
            "warrior" => Self::Warrior,
            "runner" => Self::Runner,
            "diplomat" => Self::Diplomat,
            "champion" => Self::Champion,
            "general" => Self::General,
            "stone" => Self::Stone,
            _ => return Err(UnknownPiece),
        })
    }
}

impl Icon for PieceKind {
    fn icon(&self) -> char {
        b"bwrdcgs"[*self as usize] as char
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Piece {
    pub team: Team,
    pub kind: PieceKind,
}

impl Piece {
    /// order: downward, down-side, side, up-side, up.
    ///
    /// format: (kind, maxrange)
    pub fn moves(&self) -> [(MoveKind, u8); 5] {
        let mut moves = self.kind.moves();
        if self.team == Team::Red {
            moves.swap(0, 4);
        }
        moves
    }

    pub fn ray_index(dx: i8, dy: i8) -> Option<usize> {
        Some(match [dx.abs(), dy] {
            [0, 1] => 0,
            [1, 1] => 1,
            [1, 0] => 2,
            [1, -1] => 3,
            [0, -1] => 4,
            _ => return None,
        })
    }
}

impl Icon for Piece {
    fn icon(&self) -> char {
        let mut icon = self.kind.icon();
        if self.team == Team::Blue {
            icon.make_ascii_uppercase();
        }
        icon
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tile(pub Option<Piece>);

impl TryFrom<char> for Tile {
    type Error = &'static str;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Ok(Self(match value {
            '.' | ':' | '_' => None,
            mut c => {
                let team = if c.is_ascii_uppercase() {
                    c.make_ascii_lowercase();
                    Team::Blue
                } else {
                    Team::Red
                };
                let kind = match c {
                    's' => PieceKind::Stone,
                    'b' => PieceKind::Blank,
                    'w' => PieceKind::Warrior,
                    'r' => PieceKind::Runner,
                    'd' => PieceKind::Diplomat,
                    'c' => PieceKind::Champion,
                    'g' => PieceKind::General,
                    _ => return Err("Unrecognized piece."),
                };
                Some(Piece { team, kind })
            }
        }))
    }
}

impl Icon for Tile {
    fn icon(&self) -> char {
        match self.0 {
            None => ' ',
            Some(piece) => piece.icon(),
        }
    }
}
