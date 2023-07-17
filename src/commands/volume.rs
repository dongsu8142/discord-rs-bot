use serenity::builder::*;
use serenity::model::prelude::*;
use serenity::prelude::*;
use tracing::error;

pub async fn run(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> CreateEmbed {
    let guild_id = command.guild_id.unwrap();

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        let handler_lock = manager.get(guild_id).unwrap();
        let handler = handler_lock.lock().await;
        let current = handler.queue().current();
        if current.is_none() {
            return CreateEmbed::new()
                .title("노래를 재생중이지 않습니다.")
                .colour(Colour::RED);
        };
        let option = options.get(0);
        if option.is_none() {
            let current_volume = (current.unwrap().get_info().await.unwrap().volume * 100.0) as u32;
            return CreateEmbed::new()
                .title(format!("현재 볼륨은 {}입니다", current_volume))
                .colour(Colour::BLUE);
        }
        let volume = option.unwrap().value.as_i64();
        if let Err(why) = current.unwrap().set_volume(volume.unwrap() as f32 / 100.0) {
            error!("Leave command error: {why:?}");
            return CreateEmbed::new()
                .title("볼륨 설정에 실패했습니다.")
                .colour(Colour::RED);
        }
        CreateEmbed::new()
            .title(format!("볼륨을 {}으로 설정했습니다.", volume.unwrap()))
            .colour(Colour::BLUE)
    } else {
        CreateEmbed::new()
            .title("음성 채널에 들어가 있지 않습니다.")
            .colour(Colour::RED)
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("볼륨")
        .description("재생 중인 노래의 볼륨을 설정합니다.")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::Integer,
                "볼륨",
                "설정할 볼륨을 입력해주세요",
            )
            .max_int_value(100)
            .min_int_value(0),
        )
}
