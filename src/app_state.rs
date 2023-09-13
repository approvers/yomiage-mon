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
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use crate::voice::voicevox::VoiceVoxClient;

pub struct AppState {
    pub voicevox_client: VoiceVoxClient,
    pub connected_guild_state: DashMap<GuildId, ConnectedGuildState>,
    pub subscribe_channels: HashMap<GuildId, Vec<ChannelId>>,
}

pub struct ConnectedGuildState {
    pub bound_text_channel: ChannelId,
    pub last_message_read: Option<Message>,
}

impl TypeMapKey for AppState {
    type Value = Arc<RwLock<AppState>>;
}

pub async fn initialize(client: &Client, state: AppState) {
    let mut data = client.data.write().await;
    data.insert::<AppState>(Arc::new(RwLock::new(state)));
}

pub async fn get(ctx: &Context) -> Result<Arc<RwLock<AppState>>> {
    let data = ctx.data.read().await;
    let state = data.get::<AppState>().ok_or_else(|| {
        anyhow::anyhow!("AppState is not initialized. Please call app_state::initialize() first.")
    })?;
    Ok(state.clone())
}

pub async fn add_channels(
    ctx: &Context,
    guild_id: GuildId,
    channels: Vec<ChannelId>,
) -> Result<()> {
    let state = get(ctx).await?;
    let mut state = state.write().await;
    state
        .subscribe_channels
        .entry(guild_id)
        .or_default()
        .extend(channels);
    Ok(())
}

pub async fn remove_channel(ctx: &Context, guild_id: GuildId, channel: ChannelId) -> Result<()> {
    let state = get(ctx).await?;
    let mut state = state.write().await;
    state.subscribe_channels.entry(guild_id).and_modify(|c| {
        c.retain(|c| *c != channel);
    });
    Ok(())
}

pub async fn remove_all_channels(ctx: &Context, guild_id: GuildId) -> Result<()> {
    let state = get(ctx).await?;
    let mut state = state.write().await;
    state.subscribe_channels.remove(&guild_id);
    Ok(())
}
