use serenity::builder::*;
use serenity::model::prelude::*;
use serenity::prelude::*;
use tracing::error;

pub async fn run(ctx: &Context, command: &CommandInteraction) -> CreateEmbed {
    let guild_id = command.guild_id.unwrap();

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(why) = manager.remove(guild_id).await {
            error!("Leave command error: {why:?}");
            return CreateEmbed::new()
                .description("음성 채널 나가기에 실패했습니다.")
                .colour(Colour::RED);
        }
        CreateEmbed::new()
            .description("음성 채널을 성공적으로 나갔습니다.")
            .colour(Colour::BLUE)
    } else {
        CreateEmbed::new()
            .description("음성 채널에 들어가 있지 않습니다.")
            .colour(Colour::RED)
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("나가").description("음악을 멈추고 음성 채널을 나갑니다.")
}
