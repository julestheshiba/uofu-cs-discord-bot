use poise::serenity_prelude as serenity;

use crate::{data::Data, handle_starboards::handle_starboards, text_detection::text_detection};

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    framework: poise::FrameworkContext<'_, Data, anyhow::Error>,
    _data: &Data,
) -> anyhow::Result<()> {
    match event {
        serenity::FullEvent::Message { new_message } => {
            text_detection(ctx, framework.user_data, new_message).await
        }
        serenity::FullEvent::ReactionAdd {
            add_reaction: reaction,
        }
        | serenity::FullEvent::ReactionRemove {
            removed_reaction: reaction,
        } => {
            let message = reaction.message(ctx).await?;
            handle_starboards(ctx, framework.user_data, &message, reaction).await
        }
        _ => Ok(()),
    }
}
