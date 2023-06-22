use crate::{
    client_state::{ClientState, ClientStateMap},
    config::{Context, Error},
    commands::utils
};


use log::{debug, error, warn};

/// This function uses songbird to connect the bot to the command author's voice channel.
/// The implementation assumes that the author is already in a voice channel.
pub async fn summon(context: &Context<'_>) -> Result<(), Error> {
    let guild = context.guild().unwrap();
    let guild_id = *context.guild_id().unwrap().as_u64();

    let r_lock = context.data().client_state_map.read().await;
    if r_lock.contains_key(&guild_id)
    {
        return Ok(());
    }
    drop(r_lock);

    let channel_id = guild
        .voice_states
        .get(&context.author().id)
        .and_then(|v_state| v_state.channel_id)
        .unwrap();

    if let Some(manager) = songbird::get(context.serenity_context()).await {
        manager.join(guild_id, channel_id).await.1?;
    } else {
        context.say(format!(
            "Sorry {}. I couldn't join your voice channel.\
            Please ensure that I have the permission needed to join.",
            context.author().name
        ))
        .await?;
        return Ok(());
    }

    let mut w_lock = context.data().client_state_map.write().await;
    w_lock
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
