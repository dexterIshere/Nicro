use std::{collections::HashMap, sync::Arc};

use serenity::{
    model::{
        prelude::message_component::MessageComponentInteraction,
        prelude::{ChannelId, InteractionResponseType},
    },
    prelude::Context,
};

use songbird::Call;
use tokio::sync::Mutex;

use crate::{messages::sound_board_generator, CachedSound};

pub async fn sb_pannel(
    ctx: &Context,
    channel_id: &ChannelId,
    _handler: Arc<Mutex<Call>>,
    sources: Arc<Mutex<HashMap<String, CachedSound>>>,
) {
    let _ = sound_board_generator(&ctx, *channel_id, &sources).await;
}

async fn _handle_interaction(
    ctx: &Arc<Context>,
    sources: &Arc<Mutex<HashMap<String, CachedSound>>>,
    handler: &Arc<Mutex<Call>>,
    interaction: Arc<MessageComponentInteraction>,
) {
    let cloned_sources = sources.clone();
    let sources = cloned_sources.lock().await;

    let sound = &interaction.data.custom_id;
    let source_id = sources
        .get(sound)
        .expect("Handle placed into cache at startup");

    let _ = interaction
        .create_interaction_response(ctx, |r| {
            r.kind(InteractionResponseType::ChannelMessageWithSource)
        })
        .await;
    let mut handler = handler.lock().await;

    let _ = handler.play_source(source_id.into());
}
