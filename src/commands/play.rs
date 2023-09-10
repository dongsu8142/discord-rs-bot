use crate::State;
use songbird::input::{Compose, YoutubeDl};
use std::sync::Arc;
use twilight_model::{
    application::{
        command::{Command, CommandType},
        interaction::{
            application_command::{CommandData, CommandOptionValue},
            Interaction,
        },
    },
    channel::message::Embed,
};
use twilight_util::builder::{
    command::{CommandBuilder, StringBuilder},
    embed::{EmbedAuthorBuilder, EmbedBuilder, ImageSource},
};

pub async fn run(
    interaction: Interaction,
    data: CommandData,
    state: Arc<State>,
) -> anyhow::Result<Embed> {
    let music = match &data.options.get(0).unwrap().value {
        CommandOptionValue::String(music) => music,
        _ => {
            return Ok(EmbedBuilder::new()
                .title("노래 제목을 입력해주세요.")
                .build())
        }
    };
    let guild_id = interaction.guild_id.unwrap();
    let voice_state = state
        .cache
        .voice_state(interaction.author_id().unwrap(), guild_id);
    let channel_id = match voice_state {
        Some(voice_state) => voice_state.channel_id(),
        None => {
            return Ok(EmbedBuilder::new()
                .title("음성 채널에 들어가 있지 않습니다.")
                .build())
        }
    };

    let call_lock;

    let mut call = {
        let get_call = state.songbird.get(guild_id);
        if get_call.is_some() {
            call_lock = get_call.unwrap();
            call_lock.lock().await
        } else {
            call_lock = state.songbird.join(guild_id, channel_id).await.unwrap();
            call_lock.lock().await
        }
    };

    let mut src = YoutubeDl::new(reqwest::Client::new(), format!("ytsearch1:{}", music));
    let metadata = src.aux_metadata().await.unwrap();

    call.set_bitrate(songbird::driver::Bitrate::Max);
    let _song = call.enqueue_input(src.into()).await;

    Ok(EmbedBuilder::new()
        .author(EmbedAuthorBuilder::new("노래를 재생합니다.").build())
        .title(metadata.title.as_ref().unwrap_or(&"<UNKNOWN>".to_string()))
        .url(metadata.source_url.as_ref().unwrap_or(&" ".to_string()))
        .thumbnail(ImageSource::url(
            metadata.thumbnail.as_ref().unwrap_or(&" ".to_string()),
        )?)
        .build())
}

pub fn register() -> Command {
    CommandBuilder::new("재생", "노래를 재생합니다.", CommandType::ChatInput)
        .option(StringBuilder::new("제목", "노래 제목을 입력해주세요").required(true))
        .build()
}
