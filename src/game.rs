pub mod logic;

use std::{fmt::Display, str::FromStr};

/// a single-character "icon" that an object can have
pub trait Icon {
    fn icon(&self) -> char;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Team {
    Blue,
    Red,
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
pub enum Tile {
    Empty,
    Victory,
    Piece(Piece),
}

impl TryFrom<char> for Tile {
    type Error = &'static str;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Ok(match value {
            '.' => Self::Empty,
            ':' => Self::Victory,
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
                Self::Piece(Piece { team, kind })
            }
        })
    }
}

impl Icon for Tile {
    fn icon(&self) -> char {
        match self {
            Tile::Empty => '.',
            Tile::Victory => ':',
            Tile::Piece(piece) => piece.icon(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Board {
    pub tiles: [[Tile; 10]; 10],
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
        let tiles = [(); 10].map(|_| [(); 10].map(|_| iter.next().unwrap()));
        Ok(Self { tiles })
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "   0 1 2 3 4 5 6 7 8 9")?;
        // writeln!(f, "   {}", "-".repeat(21))?;
        for (i, row) in self.tiles.iter().enumerate() {
            write!(f, "{i} ")?;
            for &tile in row {
                write!(f, "|{}", tile.icon())?;
            }
            writeln!(f, "|")?;
        }
        // writeln!(f, "   {}", "-".repeat(21))?;

        Ok(())
    }
}
