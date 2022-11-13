use std::fmt::Display;

/// a single-character "icon" that an object can have
pub trait Icon {
    fn icon(&self) -> char;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Board {
    pub tiles: [[Tile; 10]; 10],
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tile {
    Empty,
    Victory,
    Piece(Piece),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Team {
    Blue,
    Red,
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
