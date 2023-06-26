use songbird::{Event, TrackEvent};

use crate::{
    client_state::{ClientState, ClientStateMap},
    config::{Context, Error},
    handlers::{DisconnectHandler, InactivityHandler, ReconnectHandler},
    utils,
};

/// This function uses songbird to connect the bot to the command author's voice channel.
/// The implementation assumes that the author is already in a voice channel.
pub async fn summon(context: &Context<'_>) -> Result<(), Error> {
    let guild = context.guild().unwrap();
    let guild_id = *context.guild_id().unwrap().as_u64();

    let r_lock = context.data().client_state_map.read().await;
    if r_lock.contains_key(&guild_id) {
        return Ok(());
    }
    drop(r_lock);

    let channel_id = guild
        .voice_states
        .get(&context.author().id)
        .and_then(|v_state| v_state.channel_id)
        .unwrap();

    if let Some(manager) = songbird::get(context.serenity_context()).await {
        match manager.join(guild_id, channel_id).await {
            (call, Ok(_)) => {
                let mut call = call.lock().await;

                call.add_global_event(
                    Event::Core(songbird::CoreEvent::ClientDisconnect),
                    InactivityHandler {
                        client_state_map: context.data().client_state_map.clone(),
                        manager: manager.clone(),
                        cache: context.serenity_context().cache.clone(),
                        guild: guild.clone(),
                    },
                );

                call.add_global_event(
                    Event::Core(songbird::CoreEvent::DriverDisconnect),
                    DisconnectHandler {
                        client_state_map: context.data().client_state_map.clone(),
                        manager: manager.clone(),
                        guild: guild.clone(),
                    },
                );

                call.add_global_event(
                    Event::Core(songbird::CoreEvent::DriverReconnect),
                    ReconnectHandler {
                        client_state_map: context.data().client_state_map.clone(),
                        guild: guild.clone(),
                    },
                );
            }
            _ => {
                context.say("Sorry. Something went wrong.").await?;
                return Ok(());
            }
        };
    } else {
        context
            .say(format!(
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
