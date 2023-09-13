pub(crate) mod checks;
pub(crate) mod client_state;
pub(crate) mod commands;
pub(crate) mod config;
pub(crate) mod framework;
pub(crate) mod handlers;
pub(crate) mod utils;

use ::config::{Config, File, FileFormat};

use crate::config::Error;
use poise::serenity_prelude::GatewayIntents;

#[tokio::main]
async fn main() -> Result<(), Error> {
    if cfg!(debug_assertions) {
        env_logger::builder()
            .filter_module("poise", log::LevelFilter::Info)
            .filter_module(module_path!(), log::LevelFilter::Debug)
            .init();
    } else {
        env_logger::builder()
            .filter_module(module_path!(), log::LevelFilter::Warn)
            .filter_level(log::LevelFilter::Error)
            .init();
    }

    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_PRESENCES;

    let secrets = Config::builder()
        .add_source(
            if cfg!(debug_assertions) {
                File::with_name("Secrets.dev.toml")
            } else {
                File::with_name("Secrets.toml")
            }
            .format(FileFormat::Toml),
        )
        .build()
        .expect("Secrets file could not be initialized.");

    let framework = framework::build_client(secrets, intents)
        .await
        .build()
        .await
        .expect("Client initialization failed.");

    if let Err(e) = framework.start().await {
        panic!("{:?}", e);
    }

    Ok(())
}
