use super::*;

#[test]
pub(crate) fn test_diplomat() {
    let mut game = Game::from_position(
        Team::Blue,
        "
            sS........
            ..........
            .w........
            D.........
            ..........
            ..........
            ..........
            ....W..d..
            ..........
            ..........
        "
        .parse()
        .unwrap(),
    );
    game.make_move(
        game.verify_action(Move::Move {
            from: Pos(30),
            to: Pos(21),
        })
        .unwrap(),
    );
    assert_eq!(
        game.board,
        "
            sS........
            ..........
            .W........
            ..........
            ..........
            ..........
            ..........
            ....W..d..
            ..........
            ..........
        "
        .parse()
        .unwrap()
    );
    game.verify_action(Move::Move {
        from: Pos(77),
        to: Pos(74),
    })
    .unwrap_err();
    game.make_move(
        game.verify_action(Move::Move {
            from: Pos(77),
            to: Pos(47),
        })
        .unwrap(),
    );
    assert_eq!(
        game.board,
        "
            sS........
            ..........
            .W........
            ..........
            .......d..
            ..........
            ..........
            ....W.....
            ..........
            ..........
        "
        .parse()
        .unwrap()
    );
}

#[test]
pub(crate) fn test_reverse_move() {
    let game = Game::from_position(
        Team::Red,
        "
            w.........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
        "
        .parse()
        .unwrap(),
    );
    game.verify_action(Move::Move {
        from: Pos(00),
        to: Pos(10),
    })
    .unwrap_err();
}

#[test]
pub(crate) fn test_recall() {
    let mut game = Game::from_position(
        Team::Red,
        "
            w.........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
        "
        .parse()
        .unwrap(),
    );
    game.make_move(
        game.verify_action(Move::Move {
            from: Pos(00),
            to: Pos(90),
        })
        .unwrap(),
    );
    assert_eq!(
        game.board,
        "
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            w.........
        "
        .parse()
        .unwrap(),
    );
}

#[test]
pub(crate) fn test_repetition() {
    let mut game = Game::from_position(
        Team::Blue,
        "
            W........S
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            w........s
        "
        .parse()
        .unwrap(),
    );
    // go back and forth 3 times
    for _ in 0..3 {
        game.make_move(
            game.verify_action(Move::Move {
                from: Pos(00),
                to: Pos(01),
            })
            .unwrap(),
        );
        game.make_move(
            game.verify_action(Move::Move {
                from: Pos(90),
                to: Pos(91),
            })
            .unwrap(),
        );

        game.make_move(
            game.verify_action(Move::Move {
                from: Pos(01),
                to: Pos(00),
            })
            .unwrap(),
        );
        game.make_move(
            game.verify_action(Move::Move {
                from: Pos(91),
                to: Pos(90),
            })
            .unwrap(),
        );
    }
    // go back one more time
    game.make_move(
        game.verify_action(Move::Move {
            from: Pos(00),
            to: Pos(01),
        })
        .unwrap(),
    );
    // draw
    assert_eq!(game.state, GameState::Finished(Winner(None)));
}
