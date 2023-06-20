use crate::client_state::{ClientState, ClientStateMap};
use crate::config::{Context, Error};
use log::{debug, error, warn};

/// This function uses songbird to connect the bot to the command author's voice channel.
/// The implementation assumes that the author is already in a voice channel.
pub async fn summon(ctx: &mut Context<'_>) -> Result<(), Error> {
    let guild = ctx.guild().unwrap();
    let guild_id = *ctx.guild_id().unwrap().as_u64();

    let r_lock = ctx.serenity_context().data.read().await;

    if r_lock
        .get::<ClientStateMap>()
        .unwrap()
        .contains_key(&guild_id)
    {
        return Ok(());
    }
    drop(r_lock);

    let channel_id = guild
        .voice_states
        .get(&ctx.author().id)
        .and_then(|v_state| v_state.channel_id)
        .unwrap();

    if let Some(manager) = songbird::get(ctx.serenity_context()).await {
        manager.join(guild_id, channel_id).await;
    } else {
        ctx.say(format!(
            "Sorry {}. I couldn't join your voice channel.\
            Please ensure that I have the permission needed to join.",
            ctx.author().name
        ))
        .await?;
        return Err(Error::from(""));
    }
    let mut w_lock = ctx.serenity_context().data.write().await;
    w_lock
        .get_mut::<ClientStateMap>()
        .unwrap()
        .insert(
            &guild_id,
            &mut ClientState {
                current_channel: Some(*channel_id.as_u64()),
                current_track: None,
                song_queue: Some(vec![]),
                is_playing: false,
            },
        )
        .expect("ClientState insertion failed.");

    Ok(())
}
