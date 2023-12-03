use crate::{
    checks::shared_room_check,
    client_state::ClientState,
    config::{Context, Error},
};

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
