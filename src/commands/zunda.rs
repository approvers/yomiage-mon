use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::Result;

use crate::app_state;

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
#[description = "コマンドした場所を読み上げの対象にするのだ!VCに入っていないと使えないのだ!"]
#[only_in(guilds)]
async fn subscribe(ctx: &Context, msg: &Message) -> CommandResult {
    let app_state = app_state::get(ctx).await.unwrap();
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    if Some(channel_id).is_some() {
        let mut subscribe_channels = app_state.subscribe_channels.get_mut(&guild_id).unwrap();
        subscribe_channels.push(channel_id.unwrap());
        app_state
            .subscribe_channels
            .insert(guild_id, subscribe_channels.to_vec());
        check_msg(
            msg.reply(
                ctx,
                format!("{}の内容を読み上げるのだ!", channel_id.unwrap()),
            )
            .await,
        );
    } else {
        check_msg(msg.reply(ctx, "VCに入っていないのだ!").await);
    }

    Ok(())
}

#[command]
#[description = "コマンドを送った人がいるVCに入るのだ! VCに入っていないと使えないのだ!"]
#[only_in(guilds)]
async fn vc(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => {
            println!("VC connected.");
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

    let _handler = manager.join(guild_id, connect_to).await;

    Ok(())
}

#[command]
#[description = "VCから退出するのだ!自分でお話したくなったら使うのだ!"]
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
        //remove subscribe channel
        let app_state = app_state::get(ctx).await.unwrap();
        let mut subscribe_channels = app_state.subscribe_channels.get_mut(&guild_id).unwrap();
        let channel_id = guild
            .voice_states
            .get(&msg.author.id)
            .and_then(|voice_state| voice_state.channel_id);
        let index = subscribe_channels
            .iter()
            .position(|&x| x == channel_id.unwrap())
            .unwrap();
        subscribe_channels.remove(index);
        app_state
            .subscribe_channels
            .insert(guild_id, subscribe_channels.to_vec());

        check_msg(
            msg.channel_id
                .say(
                    &ctx.http,
                    "サヨナラなのだ!また必要になったら`vc`で呼ぶのだ!",
                )
                .await,
        );
    } else {
        check_msg(msg.reply(ctx, "Not in a voice channel").await);
    }

    Ok(())
}

fn check_msg(result: Result<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}
