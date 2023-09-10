use crate::State;
use std::sync::Arc;
use twilight_model::{
    application::{
        command::{Command, CommandType},
        interaction::Interaction,
    },
    channel::message::Embed,
};
use twilight_util::builder::{command::CommandBuilder, embed::EmbedBuilder};

pub async fn run(interaction: Interaction, state: Arc<State>) -> anyhow::Result<Embed> {
    let guild_id = interaction.guild_id.unwrap();

    let has_call = state.songbird.get(guild_id).is_some();

    if has_call {
        if let Err(why) = state.songbird.remove(guild_id).await {
            tracing::error!("Leave command error: {why:?}");
            return Ok(EmbedBuilder::new()
                .title("음성 채널 나가기에 실패했습니다.")
                .build());
        }
        Ok(EmbedBuilder::new()
            .title("음성 채널을 성공적으로 나갔습니다.")
            .build())
    } else {
        Ok(EmbedBuilder::new()
            .title("음성 채널에 들어가 있지 않습니다.")
            .build())
    }
}

pub fn register() -> Command {
    CommandBuilder::new(
        "나가",
        "음악을 멈추고 음성 채널을 나갑니다.",
        CommandType::ChatInput,
    )
    .build()
}
