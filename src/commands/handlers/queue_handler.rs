use songbird::{events::Event, events::EventContext, events::EventHandler, Call};

use serenity::{
    async_trait,
    prelude::{Mutex, RwLock, TypeMap},
};

use poise::serenity_prelude::GuildId;
use std::sync::Arc;

use log::warn;

use crate::client_state::{client_state::ClientState, client_state_map::ClientStateMap};

pub(crate) struct QueueHandler {
    pub(crate) guild_id: GuildId,
    pub(crate) handler: Arc<Mutex<Call>>,
    pub(crate) ctx_data: Arc<RwLock<TypeMap>>,
}

#[async_trait]
impl EventHandler for QueueHandler {
    async fn act(&self, _: &EventContext<'_>) -> Option<Event> {
        let mut lock = self.ctx_data.write().await;

        let client_map = lock.get_mut::<ClientStateMap>().unwrap();
        let client_state = client_map.get(self.guild_id.as_u64()).unwrap();
        let song_queue = client_state.clone().song_queue.unwrap();

        if song_queue.len() == 0 {
            client_map
                .update(
                    self.guild_id.as_u64(),
                    &mut ClientState {
                        is_playing: false,
                        current_track: None,
                        ..client_state.clone()
                    },
                )
                .unwrap();

            return None;
        }

        let next = song_queue.first().unwrap();

        let t_handle = self
            .handler
            .lock()
            .await
            .play_source(songbird::ytdl(next.url.clone()).await.unwrap());

        let mut updated_state = ClientState {
            is_playing: true,
            current_track: Some(t_handle),
            song_queue: Some(song_queue.clone().into_iter().skip(1).collect()),
            ..client_state.clone()
        };

        client_map
            .update(self.guild_id.as_u64(), &mut updated_state)
            .unwrap();
        None
    }
}
