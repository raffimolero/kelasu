use crate::board::Pos;
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

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
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
        let p = pieces[l];
        let nbs = [(-1, 0), (1, 0), (0, -1), (0, 1)].map(|(x, y)| p.shift(x, y));
        for i in r..pieces.len() {
            let np = pieces[i];
            if p == np {
                return Err(NonPolyomino::Duplicated);
            }
            if nbs.contains(&Some(np)) {
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

#[test]
fn test_verify_polyomino() {
    let mut pieces = [09, 10].map(Pos);
    assert_eq!(
        verify_polyomino(&mut pieces),
        Err(NonPolyomino::Disconnected),
        "You still have the wraparound bug"
    );

    let mut pieces = [09, 08, 07, 18].map(Pos);
    assert_eq!(verify_polyomino(&mut pieces), Ok(()), "T tetromino pls",);
}
