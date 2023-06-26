use crate::{
    checks::shared_room_check,
    client_state::ClientState,
    config::{Context, Error},
};

/// Commands that allow interacting with and manipulating the current track.
#[poise::command(slash_command, subcommands("pause", "resume", "info", "skip"))]
pub async fn track(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Pause the current track.
#[poise::command(slash_command, check = "shared_room_check")]
pub async fn pause(context: Context<'_>) -> Result<(), Error> {
    let guild_id = context.guild_id().unwrap();

    let mut client_map = context.data().client_state_map.write().await;

    if let Some(client_state) = client_map.get(guild_id.as_u64()).cloned() {
        match (client_state.is_playing, &client_state.current_track) {
            (true, Some(track)) => {
                track.pause()?;

                client_map
                    .update(
                        guild_id.as_u64(),
                        &mut ClientState {
                            is_playing: false,
                            ..client_state
                        },
                    )
                    .unwrap();

                context.say("Track paused.").await?;
            }
            (false, Some(_)) => {
                context.say("The track is already paused.").await?;
            }
            (_, None) => {
                context
                    .say("No tracks in the buffer. A track must be queried first")
                    .await?;
            }
        };
    } else {
        context.say("Sorry. Something has gone wrong.").await?;
    }

    Ok(())
}

/// Resume a paused track.
#[poise::command(slash_command, check = "shared_room_check")]
pub async fn resume(context: Context<'_>) -> Result<(), Error> {
    let guild_id = context.guild_id().unwrap();

    let mut client_map = context.data().client_state_map.write().await;

    if let Some(client_state) = client_map.get(guild_id.as_u64()).cloned() {
        match (client_state.is_playing, &client_state.current_track) {
            (true, Some(_)) => {
                context.say("The track is not paused.").await?;
            }
            (false, Some(track)) => {
                track.play()?;

                client_map
                    .update(
                        guild_id.as_u64(),
                        &mut ClientState {
                            is_playing: true,
                            ..client_state
                        },
                    )
                    .unwrap();

                context.say("Track resumed.").await?;
            }
            (_, None) => {
                context
                    .say("No tracks in the buffer. A track must be queried first")
                    .await?;
            }
        };
    } else {
        context.say("Sorry. Something has gone wrong.").await?;
    }

    Ok(())
}

/// See the current track's metadata.
#[poise::command(slash_command, check = "shared_room_check")]
pub async fn info(context: Context<'_>) -> Result<(), Error> {
    let guild_id = context.guild_id().unwrap();

    let client_map = context.data().client_state_map.read().await;

    if let Some(client_state) = client_map.get(guild_id.as_u64()) {
        if let Some(curr_track) = &client_state.current_track {
            let metadata = &curr_track.metadata();
            let play_status = curr_track.get_info().await.unwrap();

            let (elapsed_m, elapsed_s) = (
                play_status.play_time.as_secs() / 60,
                play_status.play_time.as_secs() % 60,
            );

            let (total_m, total_s) = (
                metadata.duration.unwrap().as_secs() / 60,
                metadata.duration.unwrap().as_secs() % 60,
            );

            context
                .say(format!(
                    "Now Playing: {} - {} [{:02}:{:02}/{:02}:{:02}]\n{}",
                    metadata.title.as_ref().unwrap(),
                    metadata.channel.as_ref().unwrap(),
                    elapsed_m,
                    elapsed_s,
                    total_m,
                    total_s,
                    metadata.source_url.as_ref().unwrap()
                ))
                .await?;
        } else {
            context.say("Nothing is currently playing.").await?;
        }

        Ok(())
    } else {
        context.say("Sorry. Something went wrong.").await?;
        Ok(())
    }
}

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
