use futures::join;

use crate::{client_state::QueueElement, config::ServerState, utils::source_retriever::SourceType};

use url::Url;

use log::warn;

const SINGLE_URI: &str = "https://youtube.com/watch?v=";
const PLAYLIST_URI: &str = "https://youtube.com/playlist?list=";

pub(crate) async fn fetch_playlist(
    playlist_id: String,
    server_state: &ServerState,
) -> Option<(QueueElement, Vec<QueueElement>)> {
    let query_builder = || {
        server_state
            .youtube_client
            .playlist_items()
            .list(&vec!["snippet".to_string()])
            .playlist_id(&playlist_id)
            .param("key", &server_state.youtube_api_key.as_str())
            .max_results(50)
    };

    let p_query = server_state
        .youtube_client
        .playlists()
        .list(&vec!["snippet".to_string()])
        .add_id(&playlist_id)
        .param("key", &server_state.youtube_api_key.as_str())
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
            url: format!("{}{}", PLAYLIST_URI, playlist_id),
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
                                SINGLE_URI,
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

pub(crate) async fn fetch_video(
    video_id: String,
    server_state: &ServerState,
) -> Option<QueueElement> {
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
        Some(QueueElement {
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
                SINGLE_URI,
                video_data.id.as_ref().unwrap().to_string()
            ),
            id: video_data.id.as_ref().unwrap().to_string(),
        })
    } else {
        None
    }
}

pub(crate) async fn handle_search_query(
    query: String,
    server_state: &ServerState,
) -> Option<SourceType> {
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

    if let Some(best_match) = result.items.clone().unwrap().first().cloned() {
        warn!("{:?}", best_match);
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
                SINGLE_URI,
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
    }
}

pub(crate) async fn process(source: &Url, server_state: &ServerState) -> Option<SourceType> {
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
            if let Some(queue) = fetch_playlist(playlist_id.to_string(), server_state).await {
                Some(SourceType::Playlist(queue))
            } else {
                None
            }
        }
        "/watch" => {
            let params = source.query_pairs();
            if params.count() > 1 {
                if let Some((_, p_id)) = params.skip_while(|(key, _)| key != "list").next() {
                    if let Some(res) = fetch_playlist(p_id.to_string(), server_state).await {
                        return Some(SourceType::Playlist(res));
                    }
                }
            }

            let video_id = match params.skip_while(|(key, _)| key != "v").next() {
                Some((_, v)) => v,
                None => return None,
            };

            if let Some(element) = fetch_video(video_id.to_string(), server_state).await {
                Some(SourceType::Single(element))
            } else {
                None
            }
        }
        _ => None,
    }
}
