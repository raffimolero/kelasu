use std::num::NonZeroU8;

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

impl Team {
    pub fn flip(&mut self) {
        *self = match self {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PieceKind {
    Stone,
    Blank,
    Warrior,
    Runner,
    Diplomat,
    Champion,
    General,
}

impl PieceKind {
    /// order: forward, fore-side, side, back-side, back.
    ///
    /// format: (kind, maxrange)
    pub fn moves(&self) -> [(MoveKind, u8); 5] {
        use MoveKind::*;
        use PieceKind::*;
        match self {
            Stone => [(MoveOnly, 0); 5],
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
        }
    }
}

impl Icon for PieceKind {
    fn icon(&self) -> char {
        b"sbwrdcg"[*self as usize] as char
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tile(pub Option<Piece>);

impl TryFrom<char> for Tile {
    type Error = &'static str;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Ok(Self(match value {
            '.' | ':' => None,
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
