use crate::models::Track;

#[derive(Clone, PartialEq, Debug)]
pub struct UserPlaylist {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub cover: Option<String>,
    pub track_ids: Vec<String>,
    pub track_count: usize,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, PartialEq, Debug)]
pub struct PlayHistory {
    pub id: String,
    pub track_id: String,
    pub played_at: i64,
}

#[derive(Clone, PartialEq, Debug)]
pub struct LikedAlbum {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub cover: Option<String>,
    pub liked_at: i64,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct UserFavorites {
    pub liked_track_ids: Vec<String>,
    pub liked_album_ids: Vec<String>,
    pub liked_artist_names: Vec<String>,
}

impl From<UserPlaylist> for Track {
    fn from(_: UserPlaylist) -> Track {
        unimplemented!("Cannot convert UserPlaylist to Track")
    }
}
