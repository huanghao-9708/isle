use crate::models::Track;

#[derive(Clone, PartialEq, Debug)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub cover: Option<String>,
    pub tracks: Vec<Track>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Clone, PartialEq, Debug)]
pub enum PlayMode {
    Sequence,
    LoopOne,
    LoopAll,
    Shuffle,
}
