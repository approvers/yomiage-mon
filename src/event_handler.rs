use serenity::model::prelude::Message;
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

        let voice_success = enqueue(&ctx, guild_id, raw_audio.into()).await;
        if voice_success.is_ok() {
            let _ = msg.react(&ctx, '💬').await;
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
            let guild_id = new_voice_state.guild_id.unwrap();
            let guild = guild_id.to_guild_cached(&ctx.cache).unwrap();
            //is bot not in voice channel
            if guild
                .voice_states
                .iter()
                .filter(|(user_id, _)| *user_id != &ctx.cache.current_user_id())
                .count()
                == 0
            {
                return;
            }
            println!("{:?}", new_voice_state.member);
            //if there is no user exxlude this bot
            //get members in voice channel
            let guild_id = new_voice_state.guild_id.unwrap();
            let guild = guild_id.to_guild_cached(&ctx.cache).unwrap();
            let manager = songbird::get(&ctx)
                .await
                .expect("Songbird Voice client placed in at initialisation.")
                .clone();
            let has_handler = manager.get(guild_id).is_some();
            let channel_id_bot_joined = guild
                .voice_states
                .get(&ctx.cache.current_user_id())
                .and_then(|voice_state| voice_state.channel_id);
            let is_members_in_vc = guild
                .voice_states
                .iter()
                .filter(|(id, _)| id != &&ctx.cache.current_user_id())
                .filter(|(_, voice_state)| {
                    voice_state.channel_id == channel_id_bot_joined
                        && !voice_state.member.clone().expect("member is none").user.bot
                })
                .count()
                > 0;
            if !has_handler && !is_members_in_vc {
                let _ = manager.remove(guild_id).await;
            }
        }
    }
}

fn is_head_symbol(text: &str) -> bool {
    let symbols = vec!["-", "!", "/", ";", "%"];
    symbols.iter().any(|symbol| text.starts_with(symbol))
}
