use crate::Context;

use poise::serenity_prelude::{self as serenity, MessageComponentInteraction};
use tracing::info;

pub async fn respond_ephemeral(
    ctx: Context<'_>,
    interaction: &MessageComponentInteraction,
    message: impl ToString,
) -> Result<(), serenity::Error> {
    let msg = message.to_string();
    // info!("responding...");
    // let Err(e) = interaction
    //     .create_interaction_response(&ctx.discord().http, |r| {
    //         r.interaction_response_data(|d| d.ephemeral(true).content(&msg))
    //     })
    //     .await
    // else {
    //     return Ok(())
    // };
    // info!("{e:?}");

    info!("following up...");
    interaction
        .create_followup_message(&ctx.discord().http, |f| f.ephemeral(true).content(&msg))
        .await?;

    Ok(())
}
