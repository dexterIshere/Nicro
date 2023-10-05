use std::{collections::HashMap, sync::Arc};

use futures::StreamExt;
use serenity::{
    model::{
        prelude::message_component::MessageComponentInteraction,
        prelude::{ChannelId, InteractionResponseType},
    },
    prelude::Context,
};

use songbird::Call;
use tokio::sync::Mutex;

use crate::{
    messages::{sb_m1, sb_m2, sb_m3, sb_m4},
    CachedSound,
};

pub async fn sb_pannel(
    ctx: &Context,
    channel_id: &ChannelId,
    handler: Arc<Mutex<Call>>,
    sources: Arc<Mutex<HashMap<String, CachedSound>>>,
) {
    let m1 = sb_m1(&ctx, *channel_id).await.unwrap();
    let m2 = sb_m2(&ctx, *channel_id).await.unwrap();
    let m3 = sb_m3(&ctx, *channel_id).await.unwrap();

    let m4 = sb_m4(&ctx, *channel_id).await.unwrap();

    let mut interaction_stream_1 = m1.await_component_interactions(&ctx).build();
    let mut interaction_stream_2 = m2.await_component_interactions(&ctx).build();
    let mut interaction_stream_3 = m3.await_component_interactions(&ctx).build();
    let mut interaction_stream_4 = m4.await_component_interactions(&ctx).build();

    let sb_ctx = Arc::new(ctx.clone());
    let src_clone = sources.clone();
    let hdlr_clone = handler.clone();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                Some(interaction) = interaction_stream_1.next() => {
                    handle_interaction(&sb_ctx, &src_clone, &hdlr_clone, interaction).await;
                },
                Some(interaction) = interaction_stream_2.next() => {
                    handle_interaction(&sb_ctx, &src_clone, &hdlr_clone, interaction).await;
                },
                Some(interaction) = interaction_stream_3.next() => {
                    handle_interaction(&sb_ctx, &src_clone,  &hdlr_clone, interaction).await;
                },
                Some(interaction) = interaction_stream_4.next() => {
                    handle_interaction(&sb_ctx, &src_clone,  &hdlr_clone, interaction).await;
                },
                else => {
                    println!("Fin des interactions");
                    break;
                }
            }
        }
    });
}

async fn handle_interaction(
    ctx: &Arc<Context>,
    sources: &Arc<Mutex<HashMap<String, CachedSound>>>,
    handler: &Arc<Mutex<Call>>,
    interaction: Arc<MessageComponentInteraction>,
) {
    let sources = sources.lock().await;

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
