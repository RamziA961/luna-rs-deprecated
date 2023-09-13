pub(crate) mod checks;
pub(crate) mod client_state;
pub(crate) mod commands;
pub(crate) mod config;
pub(crate) mod framework;
pub(crate) mod handlers;
pub(crate) mod utils;

use ::config::{Config, File, FileFormat};
use env_logger::fmt::Color;

use crate::config::Error;
use poise::serenity_prelude::GatewayIntents;

fn build_logger() -> env_logger::Builder {
    let mut log_builder = env_logger::builder();

    if cfg!(debug_assertions) {
        log_builder
            .filter_module("poise", log::LevelFilter::Info)
            .filter_module(module_path!(), log::LevelFilter::Debug)
            .filter_level(log::LevelFilter::Error)
    } else {
        log_builder
            .filter_module(module_path!(), log::LevelFilter::Warn)
            .filter_level(log::LevelFilter::Error)
    };

    log_builder.format(|buf, record| {
        use chrono::Local;
        use std::io::Write;

        let timestamp = Local::now().format("[%Y-%m-%d %H:%M:%S]");
        let level = record.level();

        let level_color = match level {
            log::Level::Error => Color::Red,
            log::Level::Warn => Color::Yellow,
            log::Level::Info => Color::Green,
            log::Level::Debug => Color::Blue,
            log::Level::Trace => Color::Magenta,
        };

        let mut timestamp_sty = buf.style();
        timestamp_sty
            .set_bg(Color::Rgb(2, 48, 32))
            .set_color(Color::White);

        let mut level_sty = buf.style();
        level_sty
            .set_color(level_color)
            .set_intense(true)
            .set_bold(true);

        write!(
            buf,
            "{} |{}|: {}\n-\n",
            timestamp_sty.value(timestamp),
            level_sty.value(level),
            record.args()
        )
    });

    log_builder
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    build_logger().init();

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
