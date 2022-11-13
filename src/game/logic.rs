use super::types::{Board, Piece, PieceKind, Team, Tile};

impl Board {
    pub fn new() -> Self {
        Self {
            tiles: [
                [Tile::Piece(Piece {
                    team: Team::Blue,
                    kind: PieceKind::Blank,
                }); 10],
                [Tile::Piece(Piece {
                    team: Team::Blue,
                    kind: PieceKind::Blank,
                }); 10],
                [
                    Tile::Piece(Piece {
                        team: Team::Blue,
                        kind: PieceKind::Stone,
                    }),
                    Tile::Empty,
                    Tile::Piece(Piece {
                        team: Team::Blue,
                        kind: PieceKind::Stone,
                    }),
                    Tile::Empty,
                    Tile::Empty,
                    Tile::Empty,
                    Tile::Empty,
                    Tile::Piece(Piece {
                        team: Team::Blue,
                        kind: PieceKind::Stone,
                    }),
                    Tile::Empty,
                    Tile::Piece(Piece {
                        team: Team::Blue,
                        kind: PieceKind::Stone,
                    }),
                ],
                [Tile::Empty; 10],
                [
                    Tile::Empty,
                    Tile::Empty,
                    Tile::Empty,
                    Tile::Empty,
                    Tile::Victory,
                    Tile::Victory,
                    Tile::Empty,
                    Tile::Empty,
                    Tile::Empty,
                    Tile::Empty,
                ],
                [
                    Tile::Empty,
                    Tile::Empty,
                    Tile::Empty,
                    Tile::Empty,
                    Tile::Victory,
                    Tile::Victory,
                    Tile::Empty,
                    Tile::Empty,
                    Tile::Empty,
                    Tile::Empty,
                ],
                [Tile::Empty; 10],
                [
                    Tile::Piece(Piece {
                        team: Team::Red,
                        kind: PieceKind::Stone,
                    }),
                    Tile::Empty,
                    Tile::Piece(Piece {
                        team: Team::Red,
                        kind: PieceKind::Stone,
                    }),
                    Tile::Empty,
                    Tile::Empty,
                    Tile::Empty,
                    Tile::Empty,
                    Tile::Piece(Piece {
                        team: Team::Red,
                        kind: PieceKind::Stone,
                    }),
                    Tile::Empty,
                    Tile::Piece(Piece {
                        team: Team::Red,
                        kind: PieceKind::Stone,
                    }),
                ],
                [Tile::Piece(Piece {
                    team: Team::Red,
                    kind: PieceKind::Blank,
                }); 10],
                [Tile::Piece(Piece {
                    team: Team::Red,
                    kind: PieceKind::Blank,
                }); 10],
            ],
        }
    }
}
