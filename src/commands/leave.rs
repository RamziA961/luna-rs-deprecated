use crate::config::{Context, Error};

use crate::checks::shared_room_check;
use crate::utils;

/// Leave the voice channel.
#[poise::command(slash_command, check = "shared_room_check")]
pub async fn leave(mut ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Bye!").await?;
    utils::banish(&mut ctx).await
}
