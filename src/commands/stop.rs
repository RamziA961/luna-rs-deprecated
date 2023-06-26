use log::warn;

use crate::{
    checks::shared_room_check,
    client_state::ClientState,
    config::{Context, Error},
};

/// Stop the current track and empty the queue.
#[poise::command(slash_command, check = "shared_room_check")]
pub(crate) async fn stop(ctx: Context<'_>) -> Result<(), Error> {
    let gid = ctx.guild_id().unwrap();

    if let Some(manager) = songbird::get(&ctx.serenity_context()).await {
        match manager.get(gid) {
            Some(handler) => {
                handler.lock().await.stop();
            }
            None => {
                ctx.say("I am not connected to any voice channel.").await?;
            }
        }
    }

    let mut client_map = ctx.data().client_state_map.write().await;
    let current_state = client_map.get(gid.as_u64()).unwrap().clone();

    let update_res = client_map.update(
        gid.as_u64(),
        &mut ClientState {
            song_queue: Some(vec![]),
            is_playing: false,
            current_track: None,
            ..current_state
        },
    );

    match update_res {
        Ok(_) => {
            ctx.say("Stopping audio and clearing queue.").await?;
            Ok(())
        }
        Err(client_error) => {
            ctx.say(
                "I have encountered some difficulties. Future queries may not behave as intended.",
            )
            .await?;
            warn!("stop::stop() encountered error: {:?}", client_error);
            Err(client_error.into())
        }
    }
}
