mod game;

use crate::game::types::Board;

fn main() {
    let board = Board::new();

    println!("{board}");
}
