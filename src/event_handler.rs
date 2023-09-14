use serenity::model::prelude::{ChannelId, Guild, Message, UserId};
use serenity::model::user::User;
use serenity::{async_trait, prelude::Context};

use serenity::model::voice::VoiceState;
use serenity::prelude::EventHandler;

use crate::app_state;
use crate::voice::call::enqueue;
use crate::voice::speech::{make_speech, SpeechRequest};
use serenity::model::gateway::Ready;
pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    async fn message(&self, ctx: Context, msg: Message) {
        let app_state = app_state::get(&ctx).await.unwrap();
        let listen_channels = app_state
            .read()
            .await
            .subscribe_channels
            .clone()
            .get(&msg.guild_id.unwrap())
            .unwrap_or(&vec![])
            .clone();
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

        if !listen_channels.contains(&msg.channel_id) {
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
        let voicevox_client = &app_state.read().await.voicevox_client;
        let eoncoded_audio = make_speech(
            voicevox_client,
            SpeechRequest {
                text: msg.clone().content,
            },
        )
        .await
        .unwrap();

        let raw_audio = eoncoded_audio.decode().await.unwrap();

        let voice_success = enqueue(&ctx, guild_id, raw_audio.into()).await;
        if voice_success.is_ok() {
            let _ = msg.react(&ctx, '💬').await;
        }
    }

    async fn voice_state_update(
        &self,
        ctx: Context,
        _old_voice_state: Option<VoiceState>,
        new_voice_state: VoiceState,
    ) {
        let guild_id = new_voice_state.guild_id.unwrap();
        let guild = guild_id.to_guild_cached(&ctx.cache).unwrap();
        let manager = songbird::get(&ctx)
            .await
            .expect("Songbird Voice client placed in at initialisation.")
            .clone();
        let channel_id_bot_joined = get_channel_id(guild.clone(), ctx.cache.current_user_id());
        let is_members_in_vc = !get_members_in_vc(
            guild,
            channel_id_bot_joined.unwrap_or_else(|| {
                panic!("Bot is not in VC. Bot is in {:?}", channel_id_bot_joined)
            }),
        )
        .is_empty();
        if !is_members_in_vc {
            let _ = manager.remove(guild_id).await;
        }
    }
}

fn is_head_symbol(text: &str) -> bool {
    let symbols = ["-", "!", "/", ";", "%"];
    symbols.iter().any(|symbol| text.starts_with(symbol))
}

fn get_channel_id(guild: Guild, user_id: UserId) -> Option<ChannelId> {
    guild
        .voice_states
        .get(&user_id)
        .and_then(|voice_state| voice_state.channel_id)
}

fn get_members_in_vc(guild: Guild, channel_id: ChannelId) -> Vec<User> {
    guild
        .voice_states
        .iter()
        .filter(|(_, voice_state)| {
            voice_state.channel_id == Some(channel_id)
                && !voice_state.member.clone().expect("member is none").user.bot
        })
        .map(|(_, voice_state)| voice_state.member.clone().expect("member is none").user)
        .collect()
}
