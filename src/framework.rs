use google_youtube3::client::auth::NoToken;

use poise::{
    framework::{Framework, FrameworkBuilder},
    serenity_prelude as serenity,
    serenity_prelude::GatewayIntents,
};

use shuttle_secrets::SecretStore;

use crate::{
    client_state::ClientStateMap,
    commands,
    config::{Error, ServerState},
};

use songbird::SerenityInit;

pub(crate) async fn build_client(
    secrets: SecretStore,
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
                commands::skip::skip(),
                commands::stop::stop(),
            ],
            ..Default::default()
        })
        .token(secrets.get("DISCORD_TOKEN").unwrap())
        .intents(intents)
        .client_settings(|cb| cb.register_songbird())
        .setup(|ctx, _, framework| {
            Box::pin(async move {
                let mut lock = ctx.data.write().await;
                lock.insert::<ClientStateMap>(ClientStateMap::new());

                let gid = secrets.get("GUILD_ID").and_then(|v| v.parse::<u64>().ok());

                let register = match gid {
                    Some(gid) => {
                        poise::builtins::register_in_guild(
                            ctx,
                            &framework.options().commands,
                            serenity::GuildId(gid),
                        )
                        .await
                    }
                    None => {
                        poise::builtins::register_globally(ctx, &framework.options().commands).await
                    }
                };

                Ok(ServerState {
                    youtube_client: yt,
                    youtube_api_key: secrets.get("YOUTUBE_API_KEY").unwrap(),
                })
            })
        })
}
