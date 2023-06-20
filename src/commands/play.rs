use futures::join;
use log::{info, warn};

use serenity::model::id::GuildId;
use songbird;
use songbird::{Event, TrackEvent};

use url;

use crate::checks::author_in_room_check;
use crate::client_state::{ClientState, ClientStateMap, QueueElement};
use crate::commands::{handlers, utils};
use crate::config::{Context, Error};

#[derive(Debug)]
pub enum PlayStatus {
    Playing(QueueElement),
    Queued(SourceType),
    PlayAndQueued(Vec<QueueElement>),
}

#[derive(Debug, Clone)]
pub enum SourceType {
    Single(QueueElement),
    Playlist((QueueElement, Vec<QueueElement>)),
}

async fn fetch_playlist(
    context: &Context<'_>,
    playlist_id: String,
) -> Option<(QueueElement, Vec<QueueElement>)> {
    let query_builder = || {
        context
            .data()
            .youtube_client
            .playlist_items()
            .list(&vec!["snippet".to_string()])
            .playlist_id(&playlist_id)
            .param("key", &context.data().youtube_api_key.as_str())
            .max_results(50)
    };

    let p_query = context
        .data()
        .youtube_client
        .playlists()
        .list(&vec!["snippet".to_string()])
        .add_id(&playlist_id)
        .param("key", &context.data().youtube_api_key.as_str())
        .max_results(1);

    if let (Ok((_, mut p_items_res)), Ok((_, p_res))) =
        join!(query_builder().doit(), p_query.doit())
    {
        let best_match = p_res
            .items
            .as_ref()
            .unwrap()
            .first()
            .as_ref()
            .unwrap()
            .snippet
            .as_ref()
            .unwrap();
        let playlist_data = QueueElement {
            title: best_match.title.as_ref().unwrap().clone(),
            channel_name: best_match
                .channel_title
                .as_ref()
                .unwrap_or(&"None".to_string())
                .clone(),
            url: format!("{}{}", "https://youtube.com/playlist?list=", playlist_id),
            id: playlist_id.clone(),
        };

        let mut playlist_elems = vec![];

        loop {
            playlist_elems = playlist_elems
                .into_iter()
                .chain(
                    p_items_res
                        .clone()
                        .items
                        .unwrap()
                        .into_iter()
                        .map(|playlist_item| QueueElement {
                            title: playlist_item.snippet.clone().unwrap().title.unwrap(),
                            channel_name: playlist_item
                                .snippet
                                .as_ref()
                                .unwrap()
                                .channel_title
                                .as_ref()
                                .unwrap()
                                .clone(),
                            url: format!(
                                "{}{}",
                                "https://youtube.com/watch?v=",
                                playlist_item
                                    .snippet
                                    .as_ref()
                                    .unwrap()
                                    .resource_id
                                    .as_ref()
                                    .unwrap()
                                    .video_id
                                    .as_ref()
                                    .unwrap()
                                    .clone()
                            ),
                            id: playlist_item
                                .snippet
                                .as_ref()
                                .unwrap()
                                .resource_id
                                .as_ref()
                                .unwrap()
                                .video_id
                                .as_ref()
                                .unwrap()
                                .clone(),
                        }),
                )
                .collect();

            if let Some(next_token) = p_items_res.clone().next_page_token {
                p_items_res = query_builder()
                    .page_token(&next_token)
                    .doit()
                    .await
                    .unwrap()
                    .1;
            } else {
                break;
            }
        }
        Some((playlist_data, playlist_elems))
    } else {
        None
    }
}

/// Attempts to retrieve a video from YouTube using a given URL or search query.
/// If successful, the function returns the video's audio and its metadata.
async fn source_input(context: &Context<'_>, query: String) -> Option<SourceType> {
    let server_state = context.data();

    // vet url to ensure it is youtube link
    // if not a url, treat as search query instead
    let source = match url::Url::parse(query.as_str()) {
        Ok(source) => {
            if source
                .domain()
                .unwrap()
                .to_lowercase()
                .contains("youtube.com")
            {
                source
            } else {
                return None;
            }
        }
        Err(_) => {
            let (_, result) = server_state
                .youtube_client
                .search()
                .list(&vec!["snippet".to_string()])
                .q(&query.as_str())
                .param("key", &server_state.youtube_api_key.as_str())
                .max_results(1)
                .doit()
                .await
                .unwrap();

            return if let Some(best_match) = result.items.clone().unwrap().first().cloned() {
                Some(SourceType::Single(QueueElement {
                    title: best_match
                        .snippet
                        .as_ref()
                        .unwrap()
                        .title
                        .as_ref()
                        .unwrap()
                        .clone(),
                    channel_name: best_match
                        .snippet
                        .as_ref()
                        .unwrap()
                        .channel_title
                        .as_ref()
                        .unwrap()
                        .clone(),
                    url: format!(
                        "{}{}",
                        "https://youtube.com/watch?v=",
                        best_match
                            .id
                            .as_ref()
                            .unwrap()
                            .video_id
                            .as_ref()
                            .unwrap()
                            .clone()
                    ),
                    id: best_match
                        .id
                        .as_ref()
                        .unwrap()
                        .video_id
                        .as_ref()
                        .unwrap()
                        .clone(),
                }))
            } else {
                None
            };
        }
    };

    // check if url points to a playlist or video
    match source.path() {
        "/playlist" => {
            let playlist_id = match source
                .query_pairs()
                .skip_while(|(key, _)| key != "list")
                .next()
            {
                Some((_, p)) => p,
                None => return None,
            };
            if let Some(queue) = fetch_playlist(context, playlist_id.to_string()).await {
                Some(SourceType::Playlist(queue))
            } else {
                None
            }
        }
        "/watch" => {
            let params = source.query_pairs();
            if params.count() > 1 {
                if let Some((_, p_id)) = params.skip_while(|(key, _)| key != "list").next() {
                    if let Some(res) = fetch_playlist(context, p_id.to_string()).await {
                        return Some(SourceType::Playlist(res));
                    }
                }
            }

            let video_id = match params.skip_while(|(key, _)| key != "v").next() {
                Some((_, v)) => v,
                None => return None,
            };

            let (_, response) = server_state
                .youtube_client
                .videos()
                .list(&vec!["snippet".to_string()])
                .add_id(&video_id)
                .param("key", &server_state.youtube_api_key.as_str())
                .doit()
                .await
                .unwrap();

            if let Some(video_data) = response.items.unwrap().first() {
                Some(SourceType::Single(QueueElement {
                    title: video_data
                        .snippet
                        .as_ref()
                        .unwrap()
                        .title
                        .as_ref()
                        .unwrap()
                        .clone(),
                    channel_name: video_data
                        .snippet
                        .as_ref()
                        .unwrap()
                        .channel_title
                        .as_ref()
                        .unwrap()
                        .clone(),
                    url: format!(
                        "{}{}",
                        "https://youtube.com/watch?v=",
                        video_data.id.as_ref().unwrap().to_string()
                    ),
                    id: video_data.id.as_ref().unwrap().to_string(),
                }))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// This function handles playing or enqueuing the requested video.
/// This includes updating the client state data and creating a
/// global event listnener to play queued videos.  
async fn handle_play(
    guild_id: &GuildId,
    ctx: &mut Context<'_>,
    input: SourceType,
) -> Result<PlayStatus, Error> {
    let manager = songbird::get(ctx.serenity_context()).await.unwrap().clone();
    let mut lock = ctx.serenity_context().data.write().await;

    let client_map = lock.get_mut::<ClientStateMap>().unwrap();
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
            handlers::QueueHandler {
                ctx_data: ctx.serenity_context().data.clone(),
                guild_id: guild_id.clone(),
                handler: handler_lock.clone(),
            },
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
#[poise::command(slash_command, check = "author_in_room_check")]
pub async fn play(
    mut context: Context<'_>,
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

    if utils::summon(&mut context).await.is_err() {
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

        match handle_play(&gid, &mut context, input).await {
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
                utils::banish(&mut context).await?;
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
