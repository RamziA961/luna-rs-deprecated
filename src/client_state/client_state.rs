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
