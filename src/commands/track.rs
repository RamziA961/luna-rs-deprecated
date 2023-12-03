use crate::config::{Context, Error};

pub(crate) mod info;
pub(crate) mod pause;
pub(crate) mod resume;
pub(crate) mod seek;
pub(crate) mod skip;

use info::info;
use pause::pause;
use resume::resume;
use seek::seek;
use skip::skip;

/// Commands that allow interacting with and manipulating the current track.
#[poise::command(slash_command, subcommands("pause", "resume", "info", "seek", "skip"))]
pub async fn track(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}
