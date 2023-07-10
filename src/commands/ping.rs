use serenity::builder::*;
use serenity::model::prelude::*;

pub fn run() -> CreateEmbed {
    CreateEmbed::new().description("Pong").colour(Colour::BLUE)
}

pub fn register() -> CreateCommand {
    CreateCommand::new("ping").description("A ping command")
}
