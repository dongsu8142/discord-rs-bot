mod commands;

use reqwest::Client as HttpClient;
use serenity::{
    async_trait,
    builder::{CreateEmbed, CreateInteractionResponseFollowup},
    model::{
        application::{Command, Interaction},
        gateway::Ready,
    },
    prelude::*,
};
use songbird::SerenityInit;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let _ = command.defer(&ctx.http).await;
            let content = match command.data.name.as_str() {
                "나가" => Some(commands::leave::run(&ctx, &command).await),
                "재생" => Some(commands::play::run(&ctx, &command, &command.data.options).await),
                "스킵" => Some(commands::skip::run(&ctx, &command).await),
                "볼륨" => {
                    Some(commands::volume::run(&ctx, &command, &command.data.options).await)
                }
                _ => Some(CreateEmbed::new().description("")),
            };

            if let Some(content) = content {
                let data = CreateInteractionResponseFollowup::new().embed(content);
                if let Err(why) = command.create_followup(&ctx.http, data).await {
                    error!("Cannot respond to slash command: {why}");
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        if let Some(shard) = ready.shard {
            info!(
                "{} is connected on shard {}/{}!",
                ready.user.name,
                shard.id.0 + 1,
                shard.total
            );
            if shard.id.0 != 0 {
                return;
            };
        }
        let commands = Command::set_global_commands(
            &ctx.http,
            vec![
                commands::leave::register(),
                commands::play::register(),
                commands::skip::register(),
                commands::volume::register(),
            ],
        )
        .await
        .expect("Error creating commands");
        for command in commands {
            let command_name = command.name;
            info!("Create command: {command_name}");
        }
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("Failed to load .env file");
    tracing_subscriber::fmt::init();
    let token = std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_VOICE_STATES;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .register_songbird()
        .type_map_insert::<commands::play::HttpKey>(HttpClient::new())
        .await
        .expect("Error creating client");
    let manager = client.shard_manager.clone();

    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(30)).await;
            let lock = manager.lock().await;
            let shard_runners = lock.runners.lock().await;
            for (id, runner) in shard_runners.iter() {
                info!(
                    "Shard ID {} is {} with a latency of {:?}",
                    id, runner.stage, runner.latency,
                );
            }
        }
    });
    tokio::spawn(async move {
        if let Err(why) = client.start_shards(2).await {
            error!("Client error: {:?}", why);
        };
    });
    let _ = tokio::signal::ctrl_c().await;
    info!("Received Ctrl-C, shutting down.");
}
