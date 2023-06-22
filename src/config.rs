use crate::client_state::client_state_map::ClientStateMap;

use google_youtube3::YouTube;
use hyper_rustls::HttpsConnector;
use hyper::client::connect::HttpConnector;

use std::sync::Arc;
use tokio::sync::RwLock;

//use derive_more::AsMut;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, ServerState, Error>;

#[derive(Clone)]
pub struct ServerState {
    pub youtube_client: YouTube<HttpsConnector<HttpConnector>>,
    pub youtube_api_key: String,
    pub client_state_map: Arc<RwLock<ClientStateMap>>
}
