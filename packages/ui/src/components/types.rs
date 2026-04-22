use api::models::Track as ApiTrackModel;
use api::Track as ApiTrack;

#[derive(Clone, PartialEq)]
pub struct Track {
    pub id: String,
    pub path: String,
    pub title: String,
    pub artist: String,
    pub artist_id: String,
    pub album: String,
    pub album_id: String,
    pub duration: String,
    pub size: String,
    pub genres: Vec<String>,
    pub cover: Option<String>,
    pub lyrics: Option<String>,
    pub played_at: Option<String>,
}

impl From<ApiTrack> for Track {
    fn from(api_track: ApiTrack) -> Self {
        let mins = api_track.duration / 60;
        let secs = api_track.duration % 60;
        let duration_str = format!("{}:{:02}", mins, secs);

        let size_mb = api_track.size as f64 / (1024.0 * 1024.0);
        let size_str = format!("{:.1} MB", size_mb);

        let played_at = api_track
            .played_at
            .map(|ts| {
                if let Some(dt) = chrono::DateTime::from_timestamp(ts, 0) {
                    let local = dt.with_timezone(&chrono::Local);
                    local.format("%Y-%m-%d %H:%M").to_string()
                } else {
                    "".to_string()
                }
            })
            .filter(|s| !s.is_empty());

        Track {
            id: api_track.id,
            path: api_track.path,
            title: api_track.title,
            artist: api_track.artist,
            artist_id: api_track.artist_id,
            album: api_track.album,
            album_id: api_track.album_id,
            duration: duration_str,
            size: size_str,
            genres: api_track.genres,
            cover: api_track.cover,
            lyrics: api_track.lyrics,
            played_at,
        }
    }
}

impl Track {
    pub fn to_api_track(&self) -> ApiTrackModel {
        let parts: Vec<&str> = self.duration.split(':').collect();
        let duration_secs: u32 = if parts.len() == 2 {
            let mins: u32 = parts[0].parse().unwrap_or(0);
            let secs: u32 = parts[1].parse().unwrap_or(0);
            mins * 60 + secs
        } else {
            self.duration.parse().unwrap_or(0)
        };

        ApiTrackModel {
            id: self.id.clone(),
            path: self.path.clone(),
            title: self.title.clone(),
            artist: self.artist.clone(),
            artist_id: self.artist_id.clone(),
            album: self.album.clone(),
            album_id: self.album_id.clone(),
            duration: duration_secs,
            size: 0,
            bitrate: None,
            extension: String::new(),
            genres: self.genres.clone(),
            added_at: 0,
            mtime: 0,
            cover: self.cover.clone(),
            lyrics: self.lyrics.clone(),
            played_at: None,
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct PlayerState {
    pub current_track: Track,
    pub is_playing: bool,
    pub progress: u32,
    pub volume: u32,
}
