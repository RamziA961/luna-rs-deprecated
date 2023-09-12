use google_youtube3::client::auth::NoToken;

use poise::{
    framework::{Framework, FrameworkBuilder},
    serenity_prelude as serenity,
    serenity_prelude::GatewayIntents,
};

use crate::{
    client_state::ClientStateMap,
    commands,
    config::{Error, ServerState},
};
use songbird::SerenityInit;

use std::sync::Arc;
use tokio::sync::RwLock;

pub(crate) async fn build_client(
    secrets: ::config::Config,
    intents: GatewayIntents,
) -> FrameworkBuilder<ServerState, Error> {
    let https = hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()
        .https_or_http()
        .enable_http1()
        .build();

    let yt = google_youtube3::YouTube::new(hyper::Client::builder().build(https), NoToken);

    Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::play::play(),
                commands::leave::leave(),
                commands::queue::queue(),
                commands::stop::stop(),
                commands::track::track(),
            ],
            ..Default::default()
        })
        .token(secrets.get::<String>("DISCORD_TOKEN").unwrap())
        .intents(intents)
        .client_settings(|cb| cb.register_songbird())
        .setup(|context, _, framework| {
            Box::pin(async move {
                let gid = secrets
                    .get("GUILD_ID")
                    .ok()
                    .and_then(|v: String| v.parse::<u64>().ok());

                match gid {
                    Some(gid) => {
                        poise::builtins::register_in_guild(
                            context,
                            &framework.options().commands,
                            serenity::GuildId(gid),
                        )
                        .await
                    }
                    None => {
                        poise::builtins::register_globally(context, &framework.options().commands)
                            .await
                    }
                }?;

                Ok(ServerState {
                    youtube_client: yt,
                    youtube_api_key: secrets.get("YOUTUBE_API_KEY").unwrap(),
                    client_state_map: Arc::new(RwLock::new(ClientStateMap::new())),
                    discord_id: context.cache.current_user_id().as_u64().clone(),
                })
            })
        })
}
