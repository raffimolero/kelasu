use kelasu_game::{
    board::{GameState, Winner},
    piece::Team,
    Game as BoardGame,
};
use poise::serenity_prelude::{self as serenity, UserId};

use crate::Context;

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
    pub async fn start(self, ctx: Context<'_>) -> Result<Winner, serenity::Error> {
        loop {
            let user = match self.game.turn {
                Team::Blue => self.blue,
                Team::Red => self.red,
            };
            ctx.say(format!("It's <@{user}>'s turn.")).await?;

            match self.game.state {
                GameState::Ongoing { draw_offered } => todo!(),
                GameState::Finished(winner) => return Ok(winner),
            }
        }
    }
}
