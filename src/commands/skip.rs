use crate::{
    checks::shared_room_check,
    config::{Context, Error},
};

/// Skip the current track.
#[poise::command(slash_command, check = "shared_room_check")]
pub async fn skip(context: Context<'_>) -> Result<(), Error> {
    let guild_id = context.guild_id().unwrap();

    let client_map = context.data().client_state_map.write().await;
    let client_state = client_map.get(guild_id.as_u64()).unwrap();

    let t_handle = match &client_state.current_track {
        Some(t_handle) => t_handle,
        None => {
            context.say("I can't skip silence.").await?;
            return Ok(());
        }
    };

    if t_handle.stop().is_err() {
        context
            .say("Sorry something went wrong. Could not skip the current track.")
            .await?;
    };

    if let Some(v) = client_state.song_queue.as_ref().unwrap_or(&vec![]).first() {
        context
            .say(format!(
                "Playing: {} by {}.\n{}",
                v.title, v.channel_name, v.url
            ))
            .await?;
    } else {
        context.say("The queue is now empty.").await?;
    }

    Ok(())
}
