use futures::join;
use log::{info, warn};

use serenity::model::id::GuildId;
use songbird;
use songbird::{Event, TrackEvent};

use url;

use crate::{
    checks::author_in_room_check,
    client_state::{ClientState, QueueElement},
    utils,
    utils::{source_retriever, source_retriever::SourceType},
    config::{Context, Error, ServerState},
    handlers::{InactivityHandler, QueueHandler}
};


#[derive(Debug)]
pub enum PlayStatus {
    Playing(QueueElement),
    Queued(SourceType),
    PlayAndQueued(Vec<QueueElement>),
}


/// Attempts to retrieve a video from YouTube using a given URL or search query.
/// If successful, the function returns the video's audio and its metadata.
async fn source_input(context: &Context<'_>, query: String) -> Option<SourceType> {
    let server_state = context.data();

    // check that domain is a supported platform
    // if not a url, treat as search query instead
    match url::Url::parse(query.as_str()) {
        Ok(source) => {
            let domain = source.domain().unwrap().to_lowercase();
            match domain {
                d if d.contains("youtube.com") => source_retriever::youtube::process(&source, server_state).await,
                d if d.contains("soundcloud.com") => todo!(),
                _ => None
            }
        }
        // Search term, handle with youtube.
        Err(_) => source_retriever::youtube::handle_search_query(query, server_state).await
    }
}

/// This function handles playing or enqueuing the requested video.
/// This includes updating the client state data and creating a
/// global event listnener to play queued videos.  
async fn handle_play(
    guild_id: &GuildId,
    ctx: &Context<'_>,
    input: SourceType,
) -> Result<PlayStatus, Error> {
    let manager = songbird::get(ctx.serenity_context()).await.unwrap().clone();
    let client_map = &mut ctx.data().client_state_map.write().await;

    let client_state = match client_map.get(guild_id.as_u64()) {
        Some(client_state) => client_state,
        None => {
            return Err(Error::from(format!(
                "ClientState for gid: {} does not exist.",
                guild_id.to_string()
            )))
        }
    };

    let (play_status, mut updated_state) = if client_state.is_playing {
        let updated_queue = client_state
            .song_queue
            .clone()
            .unwrap()
            .into_iter()
            .chain(match input.clone() {
                SourceType::Single(v) => vec![v].into_iter(),
                SourceType::Playlist((_, p)) => p.into_iter(),
            })
            .collect();

        (
            PlayStatus::Queued(input),
            ClientState {
                song_queue: Some(updated_queue),
                ..client_state.clone()
            },
        )
    } else {
        let (play_status, updated_queue) = match &input {
            SourceType::Single(v) => (
                PlayStatus::Playing(v.to_owned()),
                client_state.song_queue.to_owned(),
            ),
            SourceType::Playlist((_, p)) => {
                if client_state.song_queue.is_some() {
                    (
                        PlayStatus::PlayAndQueued(p.clone()),
                        Some(p.clone().into_iter().skip(1).collect()),
                    )
                } else {
                    (
                        PlayStatus::PlayAndQueued(p.clone()),
                        Some(
                            client_state
                                .song_queue
                                .clone()
                                .unwrap()
                                .into_iter()
                                .chain(p.clone().into_iter().skip(1))
                                .collect(),
                        ),
                    )
                }
            }
        };

        let handler_lock = manager.get_or_insert(*guild_id.as_u64());
        let mut handler = handler_lock.lock().await;

        let t_handle = match &input {
            SourceType::Single(v) => {
                handler.play_source(songbird::ytdl(v.url.clone()).await.unwrap())
            }
            SourceType::Playlist((_, p)) => handler.play_source(
                songbird::ytdl(p.first().unwrap().url.clone())
                    .await
                    .unwrap(),
            ),
        };

        handler.remove_all_global_events();
        handler.add_global_event(
            Event::Track(TrackEvent::End),
            QueueHandler {
                client_state_map: ctx.data().client_state_map.clone(),
                guild_id: guild_id.clone(),
                handler: handler_lock.clone()
            }
        );

        handler.add_global_event(
            Event::Core(songbird::CoreEvent::ClientDisconnect),
            InactivityHandler {
                client_state_map: ctx.data().client_state_map.clone(),
                guild: ctx.guild().unwrap(),
                manager: manager.clone()
            }
        );

        (
            play_status,
            ClientState {
                is_playing: true,
                current_track: Some(t_handle),
                song_queue: updated_queue,
                ..client_state.clone()
            },
        )
    };

    client_map
        .update(guild_id.as_u64(), &mut updated_state)
        .expect("Could not update client state.");

    Ok(play_status)
}

/// Summon this bot to play a YouTube video as audio.
/// Subsequent invocations enqueue requested videos.
#[poise::command(
    slash_command,
    check = "author_in_room_check"
)]
pub async fn play(
    context: Context<'_>,
    #[description = "URL or search query to the requested video."] query: Option<String>,
) -> Result<(), Error> {
    info!(
        "play::play() received query: {}.",
        &query.clone().unwrap_or_else(|| "None".to_string())
    );

    if query.is_none() {
        context
            .say("Please provide a URL of video search query.")
            .await?;
        return Ok(());
    }

    let gid = {
        match context.guild_id() {
            Some(gid) => gid,
            None => {
                warn!("play::play() could not find a valid guild id in context.");
                context.say("An unexpected error occurred.").await?;
                return Err(Error::from(""));
            }
        }
    };

    if utils::summon(&context).await.is_err() {
        warn!(
            "play::play() could not connect to voice channel in gid: {}.",
            gid.to_string()
        );
        return Ok(());
    }

    return if let (Some(input), Ok(_)) = join!(
        source_input(&context, query.clone().unwrap()),
        context.defer()
    ) {
        // respond before timeout.
        if let SourceType::Playlist((p, p_items)) = &input {
            context
                .say(format!(
                    "{} - {} with {} videos found.",
                    p.title,
                    p.channel_name,
                    p_items.len()
                ))
                .await?;
        }

        match handle_play(&gid, &context, input).await {
            Ok(play_status) => {
                context
                    .say(match play_status {
                        PlayStatus::Playing(v) => {
                            format!("Playing: {} by {}.\n{}", v.title, v.channel_name, v.url)
                        }
                        PlayStatus::Queued(st) => match st {
                            SourceType::Single(v) => {
                                format!("Queued: {} by {}.\n{}", v.title, v.channel_name, v.url)
                            }
                            SourceType::Playlist((_, p)) => format!("Queued {} videos.", p.len()),
                        },
                        PlayStatus::PlayAndQueued(p) => {
                            let v = p.first().clone().unwrap();
                            format!(
                                "Queued {} videos.\nPlaying: {} by {}.\n{}",
                                p.len(),
                                v.title,
                                v.channel_name,
                                v.url
                            )
                        }
                    })
                    .await?;

                Ok(())
            }
            Err(_) => {
                context
                    .say("Could not play the requested resource. Resetting connection...")
                    .await?;
                utils::banish(&context).await?;
                Ok(())
            }
        }
    } else {
        context
            .say(format!(
                "Could not find the requested resource: {}",
                &query.unwrap()
            ))
            .await?;
        Ok(())
    };
}
