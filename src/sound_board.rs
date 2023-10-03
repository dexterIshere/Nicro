use std::{collections::HashMap, sync::Arc};

use serenity::{
    builder::CreateButton,
    futures::StreamExt,
    model::{
        application::component::ButtonStyle,
        prelude::{message_component::MessageComponentInteraction, ReactionType},
        prelude::{ChannelId, InteractionResponseType},
    },
    prelude::Context,
};

use songbird::Call;
use tokio::sync::Mutex;

use crate::CachedSound;

fn quiz_button(name: &str, emoji: ReactionType) -> CreateButton {
    let mut b = CreateButton::default();
    b.custom_id(name);
    b.emoji(emoji);
    b.label(name);
    b.style(ButtonStyle::Primary);
    b
}

pub async fn sb_pannel(
    ctx: &Context,
    channel_id: &ChannelId,
    handler: Arc<Mutex<Call>>,
    sources: Arc<Mutex<HashMap<String, CachedSound>>>,
) {
    let content = "Voici la soundboard:";

    let m1 = channel_id
        .send_message(&ctx.http, |m| {
            m.content(content)
                .embed(|e| {
                    e.title("Soundboard")
                        .description("click pour utilser Nicro 🤖")
                })
                .components(|c| {
                    c.create_action_row(|r| {
                        r.add_button(quiz_button("goofy", "📯".parse().unwrap()));
                        r.add_button(quiz_button(
                            "benji",
                            "<:chad:1158742686858219580>".parse().unwrap(),
                        ));
                        r.add_button(quiz_button("sthu", "🪖".parse().unwrap()));
                        r.add_button(quiz_button("uwu", "🍙".parse().unwrap()));
                        r.add_button(quiz_button("so_back", "😎".parse().unwrap()))
                    })
                })
        })
        .await
        .unwrap();
    let m2 = channel_id
        .send_message(&ctx.http, |m| {
            m.content("2").components(|c| {
                c.create_action_row(|r| {
                    r.add_button(quiz_button("mario", "🪙".parse().unwrap()));
                    r.add_button(quiz_button("men_stfu", "💢".parse().unwrap()));
                    r.add_button(quiz_button("mes_bb", "🤓".parse().unwrap()));
                    r.add_button(quiz_button("lucas2", "🔞".parse().unwrap()));
                    r.add_button(quiz_button("nicro", "🤖".parse().unwrap()))
                })
            })
        })
        .await
        .unwrap();
    let m3 = channel_id
        .send_message(&ctx.http, |m| {
            m.content("3").components(|c| {
                c.create_action_row(|r| {
                    r.add_button(quiz_button("verstappen", "🏎️".parse().unwrap()));
                    r.add_button(quiz_button("whala_pardon", "🙏".parse().unwrap()));
                    r.add_button(quiz_button("c_grave", "😡".parse().unwrap()));
                    r.add_button(quiz_button("dehorsmp3", "🚪".parse().unwrap()));
                    r.add_button(quiz_button("remicaliste", "🕋".parse().unwrap()))
                })
            })
        })
        .await
        .unwrap();

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
