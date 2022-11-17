mod game;
mod util;

use crate::{game::Game, util::input};

fn main() {
    let mut game = Game::new();
    println!("{game}");

    while game.is_ongoing() {
        let p_move = game.get_move();

        let p_move = match p_move {
            Ok(x) => x,
            Err(e) => {
                println!("Move error: {e}");
                continue;
            }
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
