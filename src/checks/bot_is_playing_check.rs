use crate::config::{Context, Error};

pub async fn bot_is_playing_check(ctx: Context<'_>) -> Result<bool, Error> {
    let guild_id = ctx.guild_id().unwrap();
    let client_map = ctx.data().client_state_map.read().await;

    // this should not fail, as this check is run after shared_room_check
    let client_state = client_map.get(guild_id.as_u64()).unwrap();

    if !client_state.is_playing {
        ctx.say("Sorry but I can't do that. No tracks are currently playing.")
            .await?;
    }

    Ok(client_state.is_playing)
}
