use reqwest::Client as HttpClient;
use serenity::builder::*;
use serenity::model::prelude::*;
use serenity::prelude::*;
use songbird::input::{Compose, YoutubeDl};

pub struct HttpKey;

impl TypeMapKey for HttpKey {
    type Value = HttpClient;
}

pub async fn run(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> CreateEmbed {
    let music = options.get(0).unwrap().value.as_str().unwrap().to_string();
    let (guild_id, channel_id) = {
        let guild_id = command.guild_id.unwrap();
        let guild = ctx.cache.guild(guild_id).unwrap();
        let channel_id = guild
            .voice_states
            .get(&command.user.id)
            .and_then(|voice_state| voice_state.channel_id);
        (guild_id, channel_id)
    };

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            return CreateEmbed::new()
                .description("음성 채널에 들어가 있지 않습니다.")
                .colour(Colour::RED);
        }
    };

    let http_client = {
        let data = ctx.data.read().await;
        data.get::<HttpKey>()
            .cloned()
            .expect("Guaranteed to exist in the typemap.")
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let has_handler = manager.get(guild_id).is_some();

    let handler_lock;

    let mut handler = match has_handler {
        true => {
            handler_lock = manager.get(guild_id).unwrap();
            handler_lock.lock().await
        }
        false => {
            handler_lock = manager.join(guild_id, connect_to).await.unwrap();
            handler_lock.lock().await
        }
    };

    let mut src = YoutubeDl::new(http_client, format!("ytsearch1:{}", music));
    let metadata = src.aux_metadata().await.unwrap();

    let _ = handler.enqueue_input(src.into()).await;
    CreateEmbed::new()
        .author(CreateEmbedAuthor::new("노래를 재생합니다."))
        .title(metadata.title.as_ref().unwrap_or(&"<UNKNOWN>".to_string()))
        .url(metadata.source_url.as_ref().unwrap_or(&" ".to_string()))
        .thumbnail(metadata.thumbnail.as_ref().unwrap_or(&" ".to_string()))
        .colour(Colour::BLUE)
}

pub fn register() -> CreateCommand {
    CreateCommand::new("재생")
        .description("노래를 재생합니다.")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "제목",
                "노래 제목을 입력해주세요",
            )
            .required(true),
        )
}
