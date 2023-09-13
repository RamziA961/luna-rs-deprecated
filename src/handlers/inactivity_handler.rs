use poise::serenity_prelude::{model::guild::Guild, Cache};
use serenity::async_trait;
use songbird::{
    events::{Event, EventContext, EventHandler},
    Songbird,
};

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::client_state::ClientStateMap;

use log::{debug, error};

pub(crate) struct InactivityHandler {
    pub(crate) cache: Arc<Cache>,
    pub(crate) client_state_map: Arc<RwLock<ClientStateMap>>,
    pub(crate) guild: Guild,
    pub(crate) manager: Arc<Songbird>,
}

#[async_trait]
impl EventHandler for InactivityHandler {
    async fn act(&self, _: &EventContext<'_>) -> Option<Event> {
        let guild_id = self.guild.id;
        let mut client_map = self.client_state_map.write().await;

        debug!("Inactivity hander acting");

        if let (Some(client_state), Some(guild)) = (
            client_map.get(guild_id.as_u64()),
            self.cache.guild(guild_id),
        ) {
            if let Some(channel_id) = client_state.current_channel {
                let member_count = guild
                    .voice_states
                    .values()
                    .filter(|v_state| {
                        v_state
                            .channel_id
                            .is_some_and(|cid| cid.as_u64() == &channel_id)
                            && v_state.member.as_ref().is_some_and(|m| !m.user.bot)
                    })
                    .count();

                if member_count == 0 {
                    self.manager.remove(guild_id).await.unwrap_or_else(|err| {
                        error!("Could not leave the channel for gid: {guild_id}. Error: {err:?}");
                    });

                    client_map.remove(guild_id.as_u64()).unwrap_or_else(|err| {
                        error!("Could not update the client state map after gid {guild_id} removed. Error: {err:?}");
                    });
                }
            }
        }
        None
    }
}
