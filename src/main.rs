mod game;
mod util;

use crate::{
    game::{
        board::{GameState, Move},
        Game,
    },
    util::input,
};

fn main() {
    // HACK: changed Board::new() to be playable without merging
    let mut game = Game::new();
    println!("{game}");

    while game.is_ongoing() {
        let p_move = input("Input a move.").parse::<Move>();
        let Ok(p_move) = p_move else {
            println!("Parse error: {p_move:?}");
            continue;
        };
        let p_move = game.verify_move(p_move);
        let Ok(p_move) = p_move else {
            println!("Move error: {p_move:?}");
            continue;
        };

        println!("Move: {p_move:?}");
        if input("confirm move?")
            .chars()
            .next()
            .map(|c| c.to_ascii_lowercase())
            == Some('y')
        {
            game.make_move(p_move);
        }
        println!("{game}");
    }
}
