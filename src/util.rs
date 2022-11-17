use crate::game::board::Pos;
use std::io::{stdin, stdout, Write};

use thiserror::Error;

pub fn input(msg: &str) -> String {
    println!("{msg}");
    print!("> ");
    stdout().flush().unwrap();

    let mut buf = String::new();
    stdin().read_line(&mut buf).unwrap();
    buf.trim().to_owned()
}

#[derive(Error, Debug)]
pub enum NonPolyomino {
    #[error("Not all positions were next to each other.")]
    Disconnected,
    #[error("Some positions were duplicated.")]
    Duplicated,
}

pub fn verify_polyomino(pieces: &mut [Pos]) -> Result<(), NonPolyomino> {
    let mut l = 0;
    let mut r = 1;
    while l < r {
        let p = pieces[l].0;
        let nbs = [p - 1, p + 1, p - 10, p + 10];
        for i in r..pieces.len() {
            let np = pieces[i].0;
            if p == np {
                return Err(NonPolyomino::Duplicated);
            }
            if nbs.contains(&np) {
                pieces.swap(r, i);
                r += 1;
                if r == pieces.len() {
                    return Ok(());
                }
            }
        }
        l += 1;
    }
    Err(NonPolyomino::Disconnected)
}
