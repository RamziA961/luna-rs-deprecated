use crate::client_state::ClientStateMap;
use crate::config::{Context, Error};

pub async fn bot_in_room_check(ctx: Context<'_>) -> Result<bool, Error> {
    let guild_id = ctx.guild_id().unwrap();

    let r_lock = ctx.serenity_context().data.read().await;

    if r_lock
        .get::<ClientStateMap>()
        .unwrap()
        .contains_key(&guild_id.as_u64())
    {
        Ok(true)
    } else {
        ctx.say("Whoops. Looks like you're not in a voice channel.")
            .await?;
        Ok(false)
    }
}
