use crate::State;
use twilight_model::{
    application::{
        command::{Command, CommandType},
        interaction::Interaction,
    },
    channel::message::Embed,
};
use twilight_util::builder::{command::CommandBuilder, embed::EmbedBuilder};

pub async fn run(interaction: Interaction, state: &State) -> anyhow::Result<Embed> {
    let guild_id = interaction.guild_id.unwrap();

    let call_lock = state.songbird.get(guild_id);

    if call_lock.is_some() {
        if let Err(why) = call_lock.unwrap().lock().await.queue().skip() {
            tracing::error!("Leave command error: {why:?}");
            return Ok(EmbedBuilder::new().title("스킵에 실패했습니다.").build());
        }
        Ok(EmbedBuilder::new().title("스킵을 완료했습니다.").build())
    } else {
        Ok(EmbedBuilder::new()
            .title("음성 채널에 들어가 있지 않습니다.")
            .build())
    }
}

pub fn register() -> Command {
    CommandBuilder::new(
        "스킵",
        "재생 중인 노래를 스킵합니다.",
        CommandType::ChatInput,
    )
    .build()
}
