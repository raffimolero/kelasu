use crate::game::board::Pos;
use std::io::{stdin, stdout, Write};

pub fn input(msg: &str) -> String {
    println!("{msg}");
    print!("> ");
    stdout().flush().unwrap();

    let mut buf = String::new();
    stdin().read_line(&mut buf).unwrap();
    buf.trim().to_owned()
}

pub fn is_polyomino(pieces: &[Pos]) -> bool {
    // check for duplicates as well
    todo!()
}
