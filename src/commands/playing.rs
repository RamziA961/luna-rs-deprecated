use crate::{
    checks::shared_room_check,
    config::{Context, Error},
};

/// See the current track's metadata.
#[poise::command(slash_command, check = "shared_room_check")]
pub async fn playing(context: Context<'_>) -> Result<(), Error> {
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
