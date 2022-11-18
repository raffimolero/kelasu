use kelasu_game::{board::Winner, Game as BoardGame};
use poise::serenity_prelude::{self as serenity, UserId};

#[derive(Debug, PartialEq, Eq)]
pub enum TeamPreference {
    Blue,
    Either,
    Red,
}

#[derive(Debug)]
pub struct Game {
    pub blue: UserId,
    pub red: UserId,
    pub game: BoardGame,
}

impl Game {
    pub fn new(blue: UserId, red: UserId) -> Self {
        Self {
            blue,
            red,
            game: BoardGame::new(),
        }
    }

    /// returns Ok(Some(winner)), Ok(None) means a draw
    pub async fn start(self) -> Result<Winner, serenity::Error> {
        // TODO
        Ok(Winner(None))
    }
}
