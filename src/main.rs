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
use tracing::{error, info};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let _ = command.defer(&ctx.http).await;
            let content = match command.data.name.as_str() {
                "핑" => Some(commands::ping::run()),
                "나가" => Some(commands::leave::run(&ctx, &command).await),
                "재생" => Some(commands::play::run(&ctx, &command, &command.data.options).await),
                "스킵" => Some(commands::skip::run(&ctx, &command).await),
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
        info!("{} is connected!", ready.user.name);
        let commands = Command::set_global_commands(
            &ctx.http,
            vec![
                commands::ping::register(),
                commands::leave::register(),
                commands::play::register(),
                commands::skip::register(),
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
    dotenv::dotenv().expect("Failed to load .env file");
    tracing_subscriber::fmt::init();
    let token = std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::all();
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .register_songbird()
        .type_map_insert::<commands::play::HttpKey>(HttpClient::new())
        .await
        .expect("Error creating client");

    tokio::spawn(async move {
        let _ = client
            .start()
            .await
            .map_err(|why| error!("Client error: {why:?}"));
    });
    let _ = tokio::signal::ctrl_c().await;
    info!("Received Ctrl-C, shutting down.");
}
