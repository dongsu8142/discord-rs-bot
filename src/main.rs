mod commands;

use futures::StreamExt;
use songbird::{shards::TwilightMap, Songbird};
use std::{mem, sync::Arc};
use tracing::Level;
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{
    stream::{self, ShardEventStream},
    Config, Event, Intents, Shard,
};
use twilight_http::Client;
use twilight_model::{
    application::interaction::{application_command::CommandData, Interaction, InteractionData},
    http::interaction::{InteractionResponse, InteractionResponseType},
};

pub struct State {
    client: Client,
    cache: InMemoryCache,
    songbird: Songbird,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .compact()
        .with_max_level(Level::INFO)
        .init();
    let token = std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let client = Client::new(token.clone());
    let intents = Intents::GUILDS | Intents::GUILD_VOICE_STATES;
    let config = Config::builder(token.clone(), intents).build();
    let mut shards: Vec<Shard> =
        stream::create_recommended(&client, config, |_, builder| builder.build())
            .await?
            .collect();
    let user_id = client.current_user().await?.model().await?.id;
    let commands = [
        commands::play::register(),
        commands::skip::register(),
        commands::leave::register(),
    ];
    let application = client.current_user_application().await?.model().await?;
    let interaction_client = client.interaction(application.id);

    tracing::info!("logged as {} with ID {}", application.name, application.id);

    if let Err(error) = interaction_client.set_global_commands(&commands).await {
        tracing::error!(?error, "failed to register commands");
    }

    let cache = InMemoryCache::builder()
        .resource_types(ResourceType::VOICE_STATE)
        .build();

    let senders = TwilightMap::new(
        shards
            .iter()
            .map(|s| (s.id().number(), s.sender()))
            .collect(),
    );

    let songbird = Songbird::twilight(Arc::new(senders), user_id);

    let state = Arc::new(State {
        client,
        cache,
        songbird,
    });

    let mut stream = ShardEventStream::new(shards.iter_mut());

    while let Some((shard, event)) = stream.next().await {
        let event = match event {
            Ok(event) => event,
            Err(error) => {
                if error.is_fatal() {
                    tracing::error!(?error, "fatal error while receiving event");
                    break;
                }

                tracing::warn!(?error, "error while receiving event");
                continue;
            }
        };

        tracing::info!(kind = ?event.kind(), shard = ?shard.id().number(), "received event");
        state.songbird.process(&event).await;
        state.cache.update(&event);
        tokio::spawn(process_interactions(event, Arc::clone(&state)));
    }

    Ok(())
}

pub async fn process_interactions(event: Event, state: Arc<State>) {
    let mut interaction = match event {
        Event::InteractionCreate(interaction) => interaction.0,
        _ => return,
    };

    let data = match mem::take(&mut interaction.data) {
        Some(InteractionData::ApplicationCommand(data)) => *data,
        _ => {
            tracing::warn!("ignoring non-command interaction");
            return;
        }
    };

    if let Err(error) = state
        .client
        .interaction(interaction.application_id)
        .create_response(
            interaction.id,
            &interaction.token,
            &InteractionResponse {
                kind: InteractionResponseType::DeferredChannelMessageWithSource,
                data: None,
            },
        )
        .await
    {
        tracing::error!(?error, "error while handling command");
        return;
    };

    if let Err(error) = handle_command(interaction, data, Arc::clone(&state)).await {
        tracing::error!(?error, "error while handling command");
    }
}

async fn handle_command(
    interaction: Interaction,
    data: CommandData,
    state: Arc<State>,
) -> anyhow::Result<()> {
    let content = match &*data.name {
        "재생" => commands::play::run(interaction.clone(), data, Arc::clone(&state)).await,
        "스킵" => commands::skip::run(interaction.clone(), Arc::clone(&state)).await,
        "나가" => commands::leave::run(interaction.clone(), Arc::clone(&state)).await,
        name => anyhow::bail!("unknown command: {}", name),
    };
    if let Ok(content) = content {
        state
            .client
            .interaction(interaction.application_id)
            .create_followup(&interaction.token)
            .embeds(&[content])?
            .await?;
    }
    Ok(())
}
