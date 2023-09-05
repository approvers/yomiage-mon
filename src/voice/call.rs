use anyhow::{anyhow, Result};
use serenity::client::Context;
use songbird::{
    id::GuildId,
    input::{Codec, Container, Input, Reader},
    Call, Songbird,
};

use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn enqueue(
    ctx: &Context,
    guild_id: impl Into<GuildId>,
    raw_audio: Vec<u8>,
) -> Result<()> {
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.");
    let call = get_call(manager, guild_id).await?;

    let mut handler = call.lock().await;
    handler.play_source(Input::new(
        false,
        Reader::from_memory(raw_audio),
        Codec::Pcm,
        Container::Raw,
        None,
    ));

    Ok(())
}

async fn get_call(
    manager: Arc<Songbird>,
    guild_id: impl Into<GuildId>,
) -> Result<Arc<Mutex<Call>>> {
    let guild_id = guild_id.into();

    let call = manager
        .get(guild_id)
        .ok_or_else(|| anyhow!("Not connected to a voice channel"))?;

    Ok(call)
}
