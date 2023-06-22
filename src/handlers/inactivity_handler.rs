use poise::serenity_prelude::model::{
    guild::Guild,
    id::ChannelId
};
use songbird::{
    Songbird, 
    events::{Event, EventContext, EventHandler}
};
use serenity::async_trait;

use std::sync::Arc;
use tokio::{
    sync::{RwLock, Mutex}
};

use crate::{
    client_state::{ClientStateMap, ClientState},
    config::Context
};

use log::warn;

pub(crate) struct InactivityHandler {
    pub(crate) client_state_map: Arc<RwLock<ClientStateMap>>,
    pub(crate) manager: Arc<Songbird>,
    pub(crate) guild: Guild,
}

#[async_trait]
impl<'a> EventHandler for InactivityHandler {
    async fn act(&self, _: &EventContext<'_>) -> Option<Event> {
        let guild_id = self.guild.id;
        let mut client_map = self.client_state_map.write().await;
        
        warn!("Inactivity hander acting");
        
        if let Some(client_state) = client_map.get(guild_id.as_u64()) {
            if let Some(channel_id) = client_state.current_channel {
                
                let member_count = self
                    .guild
                    .channels
                    .get(&mut ChannelId::from(channel_id))
                    .and_then(|channel| channel.clone().guild()
                        .as_ref()
                        .and_then(|guild_channel| guild_channel.member_count)
                        .clone()
                    );
                
                if member_count == None {
                    if let Err(e) = self.manager.remove(guild_id).await {
                        warn!("ERR: {:?}", e);
                    } 
                    
                    if let Err(e) = client_map.remove(guild_id.as_u64()) {
                        warn!("ERR: {}", e);
                    }
                }
             } 
        }
        None
    }
}

