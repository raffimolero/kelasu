mod game;
mod util;

use crate::{game::Game, util::input};

fn main() {
    // HACK: changed Board::new() to be playable without merging
    let mut game = Game::new();
    println!("{game}");

    while game.is_ongoing() {
        let p_move = game.get_move();

        let Ok(p_move) = p_move else {
            println!("Move error: {p_move:?}");
            println!("Valid moves:\n\tmove <yx> to <yx>\n\tmerge <piece> at <yx> with <yx> <yx> ...");
            continue;
        };

        println!("Move: {p_move:?}");
        // if input("Confirm Move?")
        //     .chars()
        //     .next()
        //     .map(|c| c.to_ascii_lowercase())
        //     == Some('y')
        // {
        game.make_move(p_move);
        // }

        println!("{game}");
    }
}
