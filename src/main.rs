mod app_state;
mod commands;
pub mod voice;

use dashmap::DashMap;
use std::collections::HashSet;
use std::env;
use voice::voicevox::VoiceVoxClient;

use serenity::async_trait;
use serenity::framework::standard::macros::hook;
use serenity::framework::standard::{
    help_commands, Args, CommandGroup, CommandResult, HelpOptions,
};
use serenity::model::prelude::{Message, UserId};
use serenity::model::voice::VoiceState;
use serenity::{
    framework::standard::macros::{group, help},
    prelude::GatewayIntents,
};
use songbird::SerenityInit;

use commands::zunda::*;
use serenity::framework::StandardFramework;
use serenity::model::gateway::Ready;
use serenity::prelude::{Client, Context, EventHandler};
use tracing::{info, instrument};

use crate::voice::speech::{make_speech, SpeechRequest};

struct Handler;

fn load_token() -> String {
    dotenv::from_path("/run/secrets/discord_token").ok();
    env::var("TOKEN").expect("Expected a token in the environment")
}

fn load_prefix() -> String {
    env::var("PREFIX").expect("Expected a prefix in the environment")
}
#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    async fn message(&self, ctx: Context, msg: Message) {
        let guild_id = msg.guild_id.unwrap();
        let guild = guild_id.to_guild_cached(&ctx.cache).unwrap();

        let channel_id = guild
            .voice_states
            .get(&msg.author.id)
            .and_then(|voice_state| voice_state.channel_id);
        if channel_id.is_none() {
            return;
        }

        if msg.author.bot {
            return;
        }

        let voice_state = guild.voice_states.get(&msg.author.id).unwrap();
        let is_mute = voice_state.self_mute;
        if !is_mute {
            return;
        }
        println!(
            "{}({}): {}",
            msg.author.name,
            if is_mute { "mute" } else { "unmute" },
            msg.content
        );
        if is_head_symbol(&msg.content) {
            return;
        }

        let state = app_state::get(&ctx).await.unwrap();

        let eoncoded_audio = make_speech(
            &state.voicevox_client,
            SpeechRequest {
                text: msg.clone().content,
            },
        )
        .await
        .unwrap();

        let raw_audio = eoncoded_audio.decode().await.unwrap();

        let voice_success = voice::call::enqueue(&ctx, guild_id, raw_audio.into()).await;
        if !voice_success.is_err() {
            let _ = msg.react(&ctx, '👍').await;
        }
    }

    async fn voice_state_update(
        &self,
        ctx: Context,
        old_voice_state: Option<VoiceState>,
        new_voice_state: VoiceState,
    ) {
        //print voice state
        if old_voice_state.unwrap().channel_id.is_some() && new_voice_state.channel_id.is_none() {
            println!("disconnect");
            //if there is no user exxlude this bot
            //get members in voice channel
            let guild_id = new_voice_state.guild_id.unwrap();
            let guild = guild_id.to_guild_cached(&ctx.cache).unwrap();
            let manager = songbird::get(&ctx)
                .await
                .expect("Songbird Voice client placed in at initialisation.")
                .clone();
            let has_handler = manager.get(guild_id).is_some();
            let is_members_in_vc = guild
                .voice_states
                .iter()
                .filter(|(id, _)| id != &&ctx.cache.current_user_id())
                .any(|(_, voice_state)| voice_state.channel_id.is_some());
            if !is_members_in_vc && has_handler {
                let _ = manager.remove(guild_id).await;
            }
        }
    }
}

#[group]
#[description("General Commands")]
#[summary("General")]
#[commands(zunda, vc, leave)]
struct General;

#[help]
#[individual_command_tip = "これはヘルプコマンドなのだ! `-help <command>`でそれぞれのコマンドの詳細が出せるのだ!"]
#[strikethrough_commands_tip_in_guild = ""]
async fn help(
    ctx: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(ctx, msg, args, help_options, groups, owners).await;

    Ok(())
}

#[hook]
// instrument will show additional information on all the logs that happen inside
// the function.
//
// This additional information includes the function name, along with all it's arguments
// formatted with the Debug impl.
// This additional information will also only be shown if the LOG level is set to `debug`
#[instrument]
async fn before(_: &Context, msg: &Message, command_name: &str) -> bool {
    info!(
        "Got command '{}' by user '{}'",
        command_name, msg.author.name
    );

    true
}

#[tokio::main]
async fn main() {
    let token = load_token();
    let prefix = load_prefix();
    let framework = StandardFramework::new()
        .configure(|c| c.prefix(prefix.as_str()))
        .before(before)
        .help(&HELP)
        .group(&GENERAL_GROUP);
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Err creating client");

    app_state::initialize(
        &client,
        app_state::AppState {
            voicevox_client: VoiceVoxClient::new("http://voicevox:50021".to_owned()),
            connected_guild_state: DashMap::new(),
        },
    )
    .await;

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

fn is_head_symbol(text: &str) -> bool {
    let symbols = vec!["-", "!", "/", ";", "%"];
    symbols.iter().any(|symbol| text.starts_with(symbol))
}
