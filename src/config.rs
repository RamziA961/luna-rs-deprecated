use crate::client_state::client_state_map::ClientStateMap;

use google_youtube3::YouTube;
use hyper_rustls::HttpsConnector;

use hyper::client::connect::HttpConnector;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, ServerState, Error>;

pub struct ServerState {
    pub youtube_client: YouTube<HttpsConnector<HttpConnector>>,
    pub youtube_api_key: String,
}
