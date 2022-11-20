use crate::Context;

use poise::serenity_prelude::{self as serenity, MessageComponentInteraction};

pub async fn respond_ephemeral(
    ctx: Context<'_>,
    interaction: &MessageComponentInteraction,
    message: impl ToString,
) -> Result<(), serenity::Error> {
    interaction
        .create_interaction_response(&ctx.discord().http, |r| {
            r.interaction_response_data(|d| d.ephemeral(true).content(message))
        })
        .await
}
