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
    let handler = manager.get(guild_id);
    let has_handler = handler.is_some();

    if has_handler {
        if let Err(why) = handler.unwrap().lock().await.queue().skip() {
            error!("Leave command error: {why:?}");
            return CreateEmbed::new()
                .title("스킵에 실패했습니다.")
                .colour(Colour::RED);
        }
        CreateEmbed::new()
            .title("스킵을 완료했습니다.")
            .colour(Colour::BLUE)
    } else {
        CreateEmbed::new()
            .title("음성 채널에 들어가 있지 않습니다.")
            .colour(Colour::RED)
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("스킵").description("재생 중인 노래를 스킵합니다.")
}
