use std::{collections::HashMap, sync::Arc};

use futures::StreamExt;
use serenity::{
    model::{
        prelude::message_component::MessageComponentInteraction,
        prelude::{ChannelId, InteractionResponseType},
    },
    prelude::Context,
};
use std::mem;

use songbird::Call;
use tokio::sync::Mutex;

use crate::{messages::sound_board_generator, CachedSound, InteractionStreams};

pub async fn sb_pannel(
    ctx: &Context,
    channel_id: &ChannelId,
    handler: Arc<Mutex<Call>>,
    sources: Arc<Mutex<HashMap<String, CachedSound>>>,
) {
    let _ = sound_board_generator(&ctx, *channel_id, &sources).await;

    let data = ctx.data.read().await;
    let interactions_streams = data.get::<InteractionStreams>().unwrap().clone();
    let mut interaction_streams_guard = interactions_streams.lock().await;

    let old_streams = mem::take(&mut *interaction_streams_guard);

    for interaction_stream in old_streams.into_iter() {
        let sb_ctx = Arc::new(ctx.clone());
        let src_clone = sources.clone();
        let hdlr_clone = handler.clone();

        tokio::spawn(async move {
            let mut interaction_stream = interaction_stream;
            while let Some(interaction) = interaction_stream.next().await {
                handle_interaction(&sb_ctx, &src_clone, &hdlr_clone, interaction).await;
            }
        });
    }
}

async fn handle_interaction(
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
