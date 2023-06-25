use poise::serenity_prelude::model::guild::Guild;
use serenity::async_trait;
use songbird::{
    events::{Event, EventContext, EventHandler},
    Songbird,
};

use std::sync::Arc;
use tokio::sync::RwLock;

use log::warn;

use crate::client_state::{ClientState, ClientStateMap};

pub(crate) struct ReconnectHandler {
    pub(crate) client_state_map: Arc<RwLock<ClientStateMap>>,
    pub(crate) guild: Guild,
}

#[async_trait]
impl EventHandler for ReconnectHandler {
    async fn act(&self, ev: &EventContext<'_>) -> Option<Event> {
        warn!("Reconnect Handler fired.");
        let mut client_state_map = self.client_state_map.write().await;

        let ev_data = match ev {
            songbird::events::EventContext::DriverConnect(ev_data) => ev_data,
            _ => return None,
        };

        if client_state_map.get(self.guild.id.as_u64()).is_none() {
            client_state_map
                .insert(
                    self.guild.id.as_u64(),
                    &mut ClientState {
                        is_playing: false,
                        song_queue: Some(vec![]),
                        current_track: None,
                        current_channel: ev_data.channel_id.and_then(|cid| Some(cid.0)),
                    },
                )
                .unwrap();
        }

        None
    }
}
