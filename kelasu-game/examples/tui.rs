use kelasu_game::{board::Move, util::input, Game};

fn main() {
    let mut game = Game::new();
    println!("Welcome to Kelasu.");
    println!("{}", Move::SYNTAX);
    input("Press Enter to begin the game.");

    while game.is_ongoing() {
        println!("\n{game}");

        let command = input("Input a move.");
        if command == "help" {
            println!("{}", Move::SYNTAX);
            input("Press Enter to continue.");
            continue;
        }

        let p_move = game.verify_move_str(&command);
        let p_move = match p_move {
            Ok(x) => x,
            Err(e) => {
                println!("Move error: {e}");
                input("Press Enter to continue.");
                continue;
            }
        };

        println!("Move: {p_move:?}");
        if input("Confirm Move?")
            .chars()
            .next()
            .map(|c| c.to_ascii_lowercase())
            == Some('y')
        {
            game.make_move(p_move);
        }
    }
    println!("{game}");
}
