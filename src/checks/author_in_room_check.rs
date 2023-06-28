use crate::config::{Context, Error};

/// Check if the command's author is connected to a voice channel.
pub async fn author_in_room_check(ctx: Context<'_>) -> Result<bool, Error> {
    let guild = ctx.guild().unwrap();
    let author = ctx.author();

    if guild.voice_states.get(&author.id).is_none() {
        ctx.say("Whoops. Looks like you're not in a voice channel.")
            .await?;
        Ok(false)
    } else {
        Ok(true)
    }
}
