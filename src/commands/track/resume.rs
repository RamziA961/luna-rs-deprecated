use crate::{
    checks::shared_room_check,
    client_state::ClientState,
    config::{Context, Error},
};

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
