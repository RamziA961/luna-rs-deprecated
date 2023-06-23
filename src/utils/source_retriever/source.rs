use crate::client_state::QueueElement;

#[derive(Debug, Clone)]
pub(crate) enum SourceType {
    Single(QueueElement),
    Playlist((QueueElement, Vec<QueueElement>)),
}