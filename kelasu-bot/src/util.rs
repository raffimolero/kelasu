use crate::Context;

use poise::serenity_prelude::{self as serenity, MessageComponentInteraction};

pub async fn respond_ephemeral(
    ctx: Context<'_>,
    interaction: &MessageComponentInteraction,
    message: impl ToString,
) -> Result<(), serenity::Error> {
    match interaction
        .get_interaction_response(&ctx.discord().http)
        .await
    {
        Ok(_response) => {
            dbg!("following up...");
            interaction
                .create_followup_message(&ctx.discord().http, |f| {
                    f.ephemeral(true).content(message)
                })
                .await?;
        }
        Err(_) => {
            dbg!("responding...");
            interaction
                .create_interaction_response(&ctx.discord().http, |r| {
                    r.interaction_response_data(|d| d.ephemeral(true).content(message))
                })
                .await?;
        }
    }
    Ok(())
}
