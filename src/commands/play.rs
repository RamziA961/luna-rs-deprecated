use futures::join;
use log::{debug, error, info, warn, Level};

use serenity::model::id::GuildId;
use songbird::{self, Event, TrackEvent};

use url;

use crate::{
    checks::author_in_room_check,
    client_state::{ClientState, QueueElement},
    config::{Context, Error},
    handlers::QueueHandler,
    utils,
    utils::{source_retriever, source_retriever::SourceType},
};

#[derive(Debug)]
pub(crate) enum PlayStatus {
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
                _ if domain.contains("youtube.com") => {
                    source_retriever::youtube::process(&source, server_state).await
                }
                _ if domain.contains("soundcloud.com") => todo!(),
                _ => None,
            }
        }
        // Search term, handle with youtube.
        Err(_) => source_retriever::youtube::handle_search_query(query, server_state).await,
    }
}

/// This function handles playing or enqueuing the requested video.
/// This includes updating the client state data and creating an
/// event listnener to play queued videos.
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
            error!(
                "ClientState for gid: {} does not exist.",
                guild_id.to_string()
            );
            return Err(Error::from(format!(
                "ClientState for gid: {} does not exist.",
                guild_id.to_string()
            )));
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
                debug!("Initializing track.");
                let t = songbird::input::Restartable::ytdl(v.url.clone(), true).await?;
                debug!("Track initialization complete.");
                //handler.play_source(songbird::ytdl(v.url.clone()).await.unwrap())
                handler.play_source(t.into())
            }
            SourceType::Playlist((_, p)) => {
                debug!("Initializing track.");
                let t = songbird::input::Restartable::ytdl(p.first().unwrap().url.clone(), true)
                    .await?;
                debug!("Track initialization complete.");
                //let t = songbird::input::cached::Memory::new(songbird::ytdl(p.first().unwrap().url.clone()).await?).unwrap();
                // songbird::ytdl(p.first().unwrap().url.clone())
                //     .await
                //     .unwrap(),
                handler.play_source(t.into())
            }
        };
        debug!("Play called");

        if log::log_enabled!(Level::Debug) {
            let metadata = t_handle.metadata().clone();
            debug!(
                "Adding event handler for {} - {}.",
                metadata.title.map_or_else(|| "None".into(), |title| title),
                metadata
                    .channel
                    .map_or_else(|| "None".into(), |channel| channel),
            );
        }

        t_handle
            .add_event(
                Event::Track(TrackEvent::End),
                QueueHandler {
                    client_state_map: ctx.data().client_state_map.clone(),
                    guild_id: guild_id.clone(),
                    handler: handler_lock.clone(),
                },
            )
            .or_else(|err| {
                error!("Failed to add event listener for track end. Error: {err:?}");
                Err(err)
            })?;

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
        .unwrap_or_else(|err| {
            error!("Could not update the client state for {guild_id}. Error: {err:?}");
        });

    Ok(play_status)
}

/// Summon this bot to play a YouTube video as audio.
/// Subsequent invocations enqueue requested videos.
#[poise::command(slash_command, check = "author_in_room_check")]
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
                error!("play::play() could not find a valid guild id in context.");
                context.say("An unexpected error occurred.").await?;
                return Err(Error::from(
                    "play::play() could not find a valid guild id in context.",
                ));
            }
        }
    };

    if let Err(err) = utils::summon(&context).await {
        error!("play::play() could not connect to voice channel for gid: {gid}. Error: {err:?}");
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
                    "{} - {} containing {} videos found.",
                    utils::decode_html_encoded_string(&p.title),
                    utils::decode_html_encoded_string(&p.channel_name),
                    p_items.len()
                ))
                .await?;
        }

        match handle_play(&gid, &context, input).await {
            Ok(play_status) => {
                context
                    .say(match play_status {
                        PlayStatus::Playing(v) => {
                            format!(
                                "Playing: {} by {}.\n<{}>",
                                utils::decode_html_encoded_string(&v.title),
                                utils::decode_html_encoded_string(&v.channel_name),
                                v.url
                            )
                        }
                        PlayStatus::Queued(st) => match st {
                            SourceType::Single(v) => {
                                format!(
                                    "Queued: {} by {}.\n<{}>",
                                    utils::decode_html_encoded_string(&v.title),
                                    utils::decode_html_encoded_string(&v.channel_name),
                                    v.url
                                )
                            }
                            SourceType::Playlist((_, p)) => format!("Queued {} videos.", p.len()),
                        },
                        PlayStatus::PlayAndQueued(p) => {
                            let v = p.first().clone().unwrap();
                            format!(
                                "Queued {} videos.\nPlaying: {} by {}.\n<{}>",
                                p.len(),
                                utils::decode_html_encoded_string(&v.title),
                                utils::decode_html_encoded_string(&v.channel_name),
                                v.url
                            )
                        }
                    })
                    .await?;

                Ok(())
            }
            Err(err) => {
                error!("Could not play the requested resource. Error: {err:?}");
                context
                    .say("Could not play the requested resource. Resetting connection...")
                    .await?;
                utils::banish(&context).await?;
                Ok(())
            }
        }
    } else {
        warn!("Could not find the requested resource for `{query:?}`");
        context
            .say(format!(
                "Could not find the requested resource: {}",
                &query.unwrap()
            ))
            .await?;
        Ok(())
    };
}
