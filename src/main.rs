mod game;
mod util;

use crate::{
    game::{board::Move, Game},
    util::input,
};

fn main() {
    let board = Game::new();
    println!("{board}");

    let p_move = input("Input a move.").parse::<Move>();
    println!("Move: {p_move:?}");
}
