pub(crate) mod checks;
pub(crate) mod client_state;
pub(crate) mod commands;
pub(crate) mod config;
pub(crate) mod framework;

use config::{Error, ServerState};
use poise::serenity_prelude::GatewayIntents;

use shuttle_poise::ShuttlePoise;
use shuttle_secrets::SecretStore;

#[shuttle_runtime::main]
async fn launch(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> ShuttlePoise<ServerState, Error> {
    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_VOICE_STATES;

    let framework = framework::build_client(secret_store, intents)
        .await
        .build()
        .await
        .expect("Client initialization failed.");

    Ok(framework.into())
}