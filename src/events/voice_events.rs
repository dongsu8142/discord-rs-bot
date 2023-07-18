use serenity::{
    all::{ChannelId, GuildId},
    async_trait,
    prelude::Context,
};
use songbird::{Event, EventContext, EventHandler, Songbird};
use std::sync::Arc;

pub struct ChannelEmpty {
    pub ctx: Context,
    pub channel_id: ChannelId,
    pub manager: Arc<Songbird>,
    pub guild_id: GuildId,
}

#[async_trait]
impl EventHandler for ChannelEmpty {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track) = ctx {
            tracing::info!("{track:#?}");
            let member_len = self
                .ctx
                .cache
                .guild_channel(self.channel_id)
                .unwrap()
                .members(self.ctx.cache.clone())
                .unwrap()
                .len();
            if member_len == 1 {
                let _ = self.manager.remove(self.guild_id).await;
            }
        } else {
            let _ = self.manager.remove(self.guild_id).await;
        }
        None
    }
}
