use std::{collections::HashMap, sync::Arc};

use serenity::{
    builder::{CreateActionRow, CreateButton},
    model::{application::component::ButtonStyle, prelude::ChannelId},
    prelude::Context,
};
use tokio::sync::Mutex;

use crate::{CachedSound, InteractionStreams, SbMessages};

pub fn _join_msg() {}

pub fn _player_interface() {}

pub fn sb_btn(name: &str) -> CreateButton {
    let mut b = CreateButton::default();
    b.custom_id(name);
    b.label(name);
    b.style(ButtonStyle::Primary);
    b
}

pub async fn sound_board_generator(
    ctx: &Context,
    channel_id: ChannelId,
    sources: &Arc<Mutex<HashMap<String, CachedSound>>>,
) -> serenity::Result<()> {
    let clonned_sources = sources.clone();
    let sources = clonned_sources.lock().await;

    let mut btn_counter = 0;
    let mut action_row = CreateActionRow::default();

    //ids
    let data_read = ctx.data.read().await;
    let sb_msgs_ids = match data_read.get::<SbMessages>() {
        Some(value) => value,
        None => {
            println!("SbMessages non trouvé dans TypeMap.");
            return Ok(());
        }
    };
    let mut sb_msg_ids = sb_msgs_ids.lock().await;

    //interactions streams
    let int_data = ctx.data.read().await;
    let interactions_stream = match int_data.get::<InteractionStreams>() {
        Some(value) => value,
        None => {
            println!("SbMessages non trouvé dans TypeMap.");
            return Ok(());
        }
    };
    let mut interactions = interactions_stream.lock().await;

    for file_name_str in sources.keys() {
        let sb_btn = sb_btn(&file_name_str);
        action_row.add_button(sb_btn);
        btn_counter += 1;
        if btn_counter >= 5 {
            let m = channel_id
                .send_message(&ctx.http, |m| {
                    m.content("#")
                        .components(|c| c.add_action_row(action_row.clone()))
                })
                .await?;
            sb_msg_ids.insert(m.id);

            let reaction_stream = m.await_component_interactions(&ctx).build();
            interactions.push(reaction_stream);

            btn_counter = 0;
            action_row = CreateActionRow::default();
        }
    }

    if btn_counter > 0 {
        let m = channel_id
            .send_message(&ctx.http, |m| {
                m.content("#").components(|c| c.add_action_row(action_row))
            })
            .await?;
        sb_msg_ids.insert(m.id);
        let reaction_stream = m.await_component_interactions(&ctx).build();
        interactions.push(reaction_stream);
    }

    Ok(())
}
