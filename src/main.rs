mod game;

use crate::game::Board;

fn main() {
    let board = Board::new();

    println!("{board}");
}
