mod sound_board;
mod sources;
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::anyhow;
use serenity::client::Context;
use serenity::framework::standard::Args;
use serenity::prelude::TypeMapKey;
use serenity::{
    async_trait,
    client::{Client, EventHandler},
    framework::{
        standard::{
            macros::{command, group},
            CommandResult,
        },
        StandardFramework,
    },
    model::{channel::Message, gateway::Ready},
    prelude::GatewayIntents,
    Result as SerenityResult,
};

use shuttle_secrets::SecretStore;
use songbird::input::cached::Memory;
use songbird::input::Input;
use songbird::SerenityInit;
use sound_board::sb_pannel;
use sources::init_sources;
use tokio::sync::Mutex;
use tracing::{error, info};
struct Bot;

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!hello" {
            if let Err(e) = msg.channel_id.say(&ctx.http, "world!").await {
                error!("Error sending message: {:?}", e);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

pub enum CachedSound {
    // Compressed(Compressed),
    Uncompressed(Memory),
}

impl From<&CachedSound> for Input {
    fn from(obj: &CachedSound) -> Self {
        use CachedSound::*;
        match obj {
            // Compressed(c) => c.new_handle().into(),
            Uncompressed(u) => u
                .new_handle()
                .try_into()
                .expect("Failed to create decoder for Memory source."),
        }
    }
}

struct SoundStore;

impl TypeMapKey for SoundStore {
    type Value = Arc<Mutex<HashMap<String, CachedSound>>>;
}

#[group]
#[commands(join, leave, play, sb)]
struct General;

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    let token = if let Some(token) = secret_store.get("DISCORD_TOKEN") {
        token
    } else {
        return Err(anyhow!("'DISCORD_TOKEN' was not found").into());
    };

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~"))
        .group(&GENERAL_GROUP);

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let client = Client::builder(&token, intents)
        .event_handler(Bot)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;

        let audio_map_result = init_sources().await;

        match audio_map_result {
            Ok(audio_map) => {
                let audio_map_arc_mutex = Arc::new(Mutex::new(audio_map));
                data.insert::<SoundStore>(audio_map_arc_mutex);
            }
            Err(e) => {
                eprintln!("Failed to initialize sounds: {:?}", e);
            }
        }
    }

    Ok(client.into())
}

#[command]
#[only_in(guilds)]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if manager.get(guild_id).is_some() {
        check_msg(msg.reply(ctx, "Déjà dans un salon").await);
        return Ok(());
    }

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            check_msg(msg.reply(ctx, "Not in a voice channel").await);

            return Ok(());
        }
    };

    let _handler = manager.join(guild.id, connect_to).await;

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, format!("Failed: {:?}", e))
                    .await,
            );
        }

        check_msg(msg.channel_id.say(&ctx.http, "Left voice channel").await);
    } else {
        check_msg(msg.reply(ctx, "Not in a voice channel").await);
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, "Donne un vrai lien STP !")
                    .await,
            ); //provide a valid URL to a video or audio
            return Ok(());
        }
    };
    if !url.starts_with("http") {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "Must provide a valid URL")
                .await,
        );
        return Ok(());
    }
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        let source = match songbird::ytdl(&url).await {
            Ok(source) => source,
            Err(why) => {
                println!("Err starting source : {:?}", why);
                check_msg(msg.channel_id.say(&ctx.http, "Err sourcing ffmpeg").await);
                return Ok(());
            }
        };
        handler.play_source(source);
        check_msg(msg.channel_id.say(&ctx.http, "Je joue ça").await); //return a sucess mess
    } else {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "Tu n'es pas dans un salon voc")
                .await,
        );
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
async fn sb(ctx: &Context, msg: &Message) -> CommandResult {
    let sb_ctx = ctx.clone();
    let msg_ctx = msg.clone();

    tokio::spawn(async move {
        let guild = msg_ctx.guild(&sb_ctx.cache).unwrap();
        let guild_id = guild.id;

        let manager = songbird::get(&sb_ctx)
            .await
            .expect("Songbird Voice client placed in at initialisation.")
            .clone();

        if let Some(handler_lock) = manager.get(guild_id) {
            let handler = handler_lock.clone();

            let data_read = sb_ctx.data.read().await;
            let sources = data_read
                .get::<SoundStore>()
                .cloned()
                .expect("Sound cache was installed at startup.");

            sb_pannel(&sb_ctx, &msg_ctx.channel_id, handler, sources).await;
        } else {
            check_msg(
                msg_ctx
                    .channel_id
                    .say(&sb_ctx.http, "Not in a voice channel to play in")
                    .await,
            );
        }
    });

    Ok(())
}

fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}
