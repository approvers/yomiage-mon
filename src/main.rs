mod app_state;
mod commands;
mod event_handler;
pub mod voice;

use dashmap::DashMap;
use event_handler::Handler;
use std::collections::HashSet;
use std::env;
use voice::voicevox::VoiceVoxClient;

use serenity::framework::standard::macros::hook;
use serenity::framework::standard::{
    help_commands, Args, CommandGroup, CommandResult, HelpOptions,
};
use serenity::model::prelude::{Message, UserId};
use serenity::{
    framework::standard::macros::{group, help},
    prelude::GatewayIntents,
};
use songbird::SerenityInit;

use commands::zunda::*;
use serenity::framework::StandardFramework;
use serenity::prelude::{Client, Context};
use tracing::{info, instrument};

fn load_token() -> String {
    dotenv::from_path("/run/secrets/discord_token").ok();
    env::var("TOKEN").expect("Expected a token in the environment")
}

fn load_prefix() -> String {
    env::var("PREFIX").expect("Expected a prefix in the environment")
}

#[group]
#[description("General Commands")]
#[summary("General")]
#[commands(zunda, subscribe, vc, leave)]
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
            subscribe_channels: DashMap::new(),
        },
    )
    .await;

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
