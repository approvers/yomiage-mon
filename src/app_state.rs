use anyhow::Result;
use dashmap::DashMap;
use serenity::{
    client::{Client, Context},
    model::{
        channel::Message,
        id::{ChannelId, GuildId},
    },
    prelude::TypeMapKey,
};
use std::sync::Arc;

use crate::voice::voicevox::VoiceVoxClient;

pub struct AppState {
    pub voicevox_client: VoiceVoxClient,
    pub connected_guild_state: DashMap<GuildId, ConnectedGuildState>,
    pub subscribe_channels: DashMap<GuildId, Vec<ChannelId>>,
}

pub struct ConnectedGuildState {
    pub bound_text_channel: ChannelId,
    pub last_message_read: Option<Message>,
}

impl TypeMapKey for AppState {
    type Value = Arc<AppState>;
}

pub async fn initialize(client: &Client, state: AppState) {
    let mut data = client.data.write().await;
    data.insert::<AppState>(Arc::new(state));
}

pub async fn get(ctx: &Context) -> Result<Arc<AppState>> {
    let data = ctx.data.read().await;

    let state_ref = data
        .get::<AppState>()
        .ok_or_else(|| anyhow::anyhow!("AppState is not initialized"))?;

    Ok(state_ref.clone())
}
