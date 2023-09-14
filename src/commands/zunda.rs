use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::Result;

use crate::app_state::{self, add_channels, remove_all_channels, remove_channel};

#[command]
#[description = "Zunda!"]
async fn zunda(ctx: &Context, msg: &Message) -> CommandResult {
    println!("zunda command");
    msg.channel_id
        .say(
            &ctx.http,
            format!("{}, おはようなのだ!", msg.author.mention()),
        )
        .await?;

    Ok(())
}

#[command]
#[description = "コマンドを送った人がいるVCに入るのだ! VCに入っていないと使えないのだ!"]
#[only_in(guilds)]
async fn vc(ctx: &Context, msg: &Message) -> CommandResult {
    let app_state = app_state::get(ctx).await.unwrap();

    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => {
            println!("VC connected.");
            let _ = add_channels(ctx, guild_id, vec![channel, msg.channel_id]).await;
            let state = app_state.read().await;
            check_msg(
                msg.reply(
                    ctx,
                    format!(
                        "VCに入ったのだ! 読み上げ対象は\n {} \nなのだ!",
                        state
                            .subscribe_channels
                            .get(&msg.guild_id.unwrap())
                            .unwrap_or(&vec![])
                            .iter()
                            .map(|c| format!(" <#{}> ", c))
                            .collect::<Vec<String>>()
                            .join("\n")
                    ),
                )
                .await,
            );
            channel
        }
        None => {
            check_msg(msg.reply(ctx, "VCに入っていないのだ!").await);

            return Ok(());
        }
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let handler = manager.join(guild_id, connect_to).await;
    match handler.1 {
        Ok(_) => {
            println!("Joined VC");
        }
        Err(why) => {
            println!("Failed to join VC: {:?}", why);
            check_msg(
                msg.channel_id
                    .say(&ctx.http, format!("Error joining the channel: {:?}", why))
                    .await,
            );
        }
    }

    Ok(())
}

#[command]
#[description = "VCから退出するのだ!自分でお話したくなったら使うのだ!"]
#[only_in(guilds)]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;
    let has_handler = has_handler(ctx, guild_id).await;
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, format!("Failed: {:?}", e))
                    .await,
            );
        }

        if let Err(e) = remove_all_channels(ctx, guild_id).await {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, format!("Failed: {:?}", e))
                    .await,
            );
        } else {
            check_msg(
                msg.channel_id
                    .say(
                        &ctx.http,
                        "サヨナラなのだ!また必要になったら`vc`で呼ぶのだ!",
                    )
                    .await,
            );
        }
    } else {
        check_msg(msg.reply(ctx, "Not in a voice channel").await);
    }

    Ok(())
}

#[command]
#[aliases("add")]
#[description = "読み上げ対象のチャンネルを追加するのだ!"]
#[only_in(guilds)]
async fn listen(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    if has_handler(ctx, guild_id).await {
        if let Err(e) = add_channels(ctx, guild_id, vec![msg.channel_id]).await {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, format!("Failed: {:?}", e))
                    .await,
            );
        } else {
            check_msg(msg.reply(ctx, "VCの読み上げ対象に追加したのだ!").await);
        }
    }

    Ok(())
}

#[command]
#[description = "読み上げ対象のチャンネルを確認するのだ!"]
#[only_in(guilds)]
async fn list(ctx: &Context, msg: &Message) -> CommandResult {
    let app_state = app_state::get(ctx).await.unwrap();
    let subscribe_channels = app_state.read().await.subscribe_channels.clone();

    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    if let Some(channels) = subscribe_channels.get(&guild_id) {
    check_msg(
        msg.reply(
            ctx,
            format!(
                "読み上げ対象は\n {} \nなのだ!",
                channels
                    .iter()
                    .map(|c| format!(" <#{}> ", c))
                    .collect::<Vec<String>>()
                    .join("\n")
            ),
        )
        .await,
    );
    } else {
        check_msg(msg.reply(ctx, "読み上げ対象はないのだ!").await);
        return Ok(());
    }

    Ok(())
}

#[command]
#[aliases("remove")]
#[description = "読み上げ対象のチャンネルを削除するのだ!"]
#[only_in(guilds)]
async fn listen_remove(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    if has_handler(ctx, guild_id).await {
        if let Err(e) = remove_channel(ctx, guild_id, msg.channel_id).await {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, format!("Failed: {:?}", e))
                    .await,
            );
        } else {
            check_msg(msg.reply(ctx, "VCの読み上げ対象から削除したのだ!").await);
        }
    }

    Ok(())
}

async fn has_handler(ctx: &Context, guild_id: GuildId) -> bool {
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();
    manager.get(guild_id).is_some()
}

fn check_msg(result: Result<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}
