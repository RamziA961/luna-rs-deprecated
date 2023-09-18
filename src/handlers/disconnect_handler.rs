use poise::serenity_prelude::model::guild::Guild;
use serenity::async_trait;
use songbird::{
    events::{Event, EventContext, EventHandler},
    Songbird,
};

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{client_state::ClientStateMap, logging::Log};

pub(crate) struct DisconnectHandler {
    pub(crate) client_state_map: Arc<RwLock<ClientStateMap>>,
    pub(crate) manager: Arc<Songbird>,
    pub(crate) guild: Guild,
}

impl Log for DisconnectHandler {
    fn log(&self) {
        use log::info;
        info!("DisconnectHandler({}) event fired.", self.guild.id);
    }
}

#[async_trait]
impl EventHandler for DisconnectHandler {
    async fn act(&self, _: &EventContext<'_>) -> Option<Event> {
        let mut client_map = self.client_state_map.write().await;

        if client_map.get(self.guild.id.as_u64()).is_some() {
            self.manager.remove(self.guild.id).await.unwrap();
            client_map.remove(self.guild.id.as_u64()).unwrap();
        }

        None
    }
}
