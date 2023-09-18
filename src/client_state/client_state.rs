use std::fmt::Display;

use songbird::tracks::TrackHandle;

#[derive(Default, Debug, Clone)]
pub struct ClientState {
    pub(crate) is_playing: bool,
    pub(crate) current_channel: Option<u64>,
    pub(crate) current_track: Option<TrackHandle>,
    pub(crate) song_queue: Option<Vec<QueueElement>>,
}

impl PartialEq for ClientState {
    fn eq(&self, other: &Self) -> bool {
        self.current_channel == other.current_channel
    }
}

impl Eq for ClientState {}

#[derive(Debug, Clone)]
pub struct QueueElement {
    pub(crate) title: String,
    pub(crate) channel_name: String,
    pub(crate) url: String,
    pub(crate) id: String,
}

impl Display for ClientState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let track_data = self.current_track.as_ref().map_or_else(
            || "None".to_string(),
            |t| {
                format!(
                    "<url: {}>",
                    t.metadata().source_url.as_ref().clone().unwrap()
                )
            },
        );

        write!(
            f,
            "ClientState(channel: {}, est_alloc: {} B)::{{ is_playing: {}, current_track: {}, song_queue: {:?}}} ",
            self.current_channel.map_or_else(|| "None".to_string(), |c| format!("{}", c)),
            std::mem::size_of_val(self),
            self.is_playing,
            track_data,
            self.song_queue
        )
    }
}
