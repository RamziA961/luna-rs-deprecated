use crate::client_state::ClientStateMap;
use crate::config::{Context, Error};

/// Check if the command's author is in the same voice channel as the bot.
/// Implicitly carries out a check to ensure that the author is connected any voice channel.
pub async fn shared_room_check(ctx: Context<'_>) -> Result<bool, Error> {
    let guild = ctx.guild().unwrap();
    let guild_id = ctx.guild_id().unwrap();

    let author = ctx.author();

    let auth_vc_id = match guild
        .voice_states
        .get(&author.id)
        .and_then(|vc| vc.channel_id)
    {
        Some(vc) => vc,
        None => {
            ctx.say("Whoops. It looks like you're not in a voice channel.")
                .await?;
            return Ok(false);
        }
    };

    let client_map = ctx.data().client_state_map.read().await;


    let client_state = match client_map.get(guild_id.as_u64()) {
        Some(client_state) => client_state,
        None => return Ok(false),
    };

    if client_state.current_channel.is_some()
        && client_state.current_channel.unwrap() == *auth_vc_id.as_u64()
    {
        Ok(true)
    } else {
        ctx.say(
            "Seems that you're in a different voice channel.\
            You can only issue commands if you are in the same voice channel.",
        )
        .await?;

        Ok(false)
    }
}
