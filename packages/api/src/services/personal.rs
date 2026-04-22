use crate::data::Store;
use crate::models::{
    Artist, LikedAlbum, PaginatedResult, PlayHistory, Track, UserFavorites, UserPlaylist,
};
use chrono::Utc;
use std::path::PathBuf;
use uuid::Uuid;

pub struct PersonalService {
    store: Option<Store>,
}

impl PersonalService {
    pub fn new() -> Self {
        PersonalService { store: None }
    }

    pub fn init_database(&mut self, db_path: PathBuf) -> Result<(), String> {
        let store = Store::new(db_path).map_err(|e| e.to_string())?;
        self.store = Some(store);
        Ok(())
    }

    pub fn like_track(&self, track_id: &str) -> Result<(), String> {
        let store = self.store.as_ref().ok_or("Database not initialized")?;
        store.like_track(track_id).map_err(|e| e.to_string())
    }

    pub fn unlike_track(&self, track_id: &str) -> Result<(), String> {
        let store = self.store.as_ref().ok_or("Database not initialized")?;
        store.unlike_track(track_id).map_err(|e| e.to_string())
    }

    pub fn is_track_liked(&self, track_id: &str) -> bool {
        let store = match self.store.as_ref() {
            Some(s) => s,
            None => return false,
        };
        store.is_track_liked(track_id).unwrap_or(false)
    }

    pub fn get_liked_tracks(&self) -> Result<Vec<Track>, String> {
        let res = self.get_liked_tracks_paginated(10000, 0)?;
        Ok(res.items)
    }

    pub fn get_liked_tracks_paginated(
        &self,
        page_size: usize,
        offset: usize,
    ) -> Result<PaginatedResult<Track>, String> {
        let store = self.store.as_ref().ok_or("Database not initialized")?;
        let (items, total) = store
            .get_liked_tracks_paginated(page_size, offset)
            .map_err(|e| e.to_string())?;

        Ok(PaginatedResult {
            items,
            total,
            page: (offset / page_size) + 1,
            page_size,
        })
    }

    pub fn create_playlist(
        &self,
        name: &str,
        description: &str,
        tags: Vec<String>,
    ) -> Result<UserPlaylist, String> {
        let store = self.store.as_ref().ok_or("Database not initialized")?;
        let now = Utc::now().timestamp();
        let playlist = UserPlaylist {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: description.to_string(),
            tags,
            cover: None,
            track_ids: Vec::new(),
            track_count: 0,
            created_at: now,
            updated_at: now,
        };
        store
            .insert_playlist(&playlist)
            .map_err(|e| e.to_string())?;
        Ok(playlist)
    }

    pub fn get_all_playlists(&self) -> Result<Vec<UserPlaylist>, String> {
        let store = self.store.as_ref().ok_or("Database not initialized")?;
        store.get_all_playlists().map_err(|e| e.to_string())
    }

    pub fn get_playlist(&self, id: &str) -> Result<Option<UserPlaylist>, String> {
        let store = self.store.as_ref().ok_or("Database not initialized")?;
        store.get_playlist(id).map_err(|e| e.to_string())
    }

    pub fn add_track_to_playlist(&self, playlist_id: &str, track_id: &str) -> Result<(), String> {
        let store = self.store.as_ref().ok_or("Database not initialized")?;
        store
            .add_track_to_playlist(playlist_id, track_id)
            .map_err(|e| e.to_string())?;
        self.refresh_playlist_cover(store, playlist_id)
    }

    pub fn add_tracks_to_playlist_batch(
        &self,
        playlist_id: &str,
        track_ids: &[String],
    ) -> Result<(), String> {
        let store = self.store.as_ref().ok_or("Database not initialized")?;
        if track_ids.is_empty() {
            return Ok(());
        }
        store
            .add_tracks_to_playlist_batch(playlist_id, track_ids)
            .map_err(|e| e.to_string())?;
        self.refresh_playlist_cover(store, playlist_id)
    }

    pub fn remove_track_from_playlist(
        &self,
        playlist_id: &str,
        track_id: &str,
    ) -> Result<(), String> {
        let store = self.store.as_ref().ok_or("Database not initialized")?;
        store
            .remove_track_from_playlist(playlist_id, track_id)
            .map_err(|e| e.to_string())?;
        self.refresh_playlist_cover(store, playlist_id)
    }

    pub fn delete_playlist(&self, id: &str) -> Result<(), String> {
        let store = self.store.as_ref().ok_or("Database not initialized")?;
        store.delete_playlist(id).map_err(|e| e.to_string())
    }

    pub fn update_playlist(
        &self,
        id: &str,
        name: &str,
        description: &str,
        tags: Vec<String>,
    ) -> Result<(), String> {
        let store = self.store.as_ref().ok_or("Database not initialized")?;
        store
            .update_playlist(id, name, description, &tags)
            .map_err(|e| e.to_string())
    }

    fn refresh_playlist_cover(&self, store: &Store, playlist_id: &str) -> Result<(), String> {
        if let Ok(Some(mut playlist)) = store.get_playlist(playlist_id) {
            if let Some(first_track_id) = playlist.track_ids.first() {
                if let Ok(Some(track)) = store.get_track(first_track_id) {
                    playlist.cover = track.cover;
                    return store
                        .update_playlist_cover(playlist_id, &playlist.cover)
                        .map_err(|e| e.to_string());
                }
            }
        }
        Ok(())
    }

    pub fn get_playlist_tracks(&self, playlist_id: &str) -> Result<Vec<Track>, String> {
        let res = self.get_playlist_tracks_paginated(playlist_id, 10000, 0)?;
        Ok(res.items)
    }

    pub fn get_playlist_tracks_paginated(
        &self,
        playlist_id: &str,
        page_size: usize,
        offset: usize,
    ) -> Result<PaginatedResult<Track>, String> {
        let store = self.store.as_ref().ok_or("Database not initialized")?;
        let (items, total) = store
            .get_playlist_tracks_paginated(playlist_id, page_size, offset)
            .map_err(|e| e.to_string())?;

        Ok(PaginatedResult {
            items,
            total,
            page: (offset / page_size) + 1,
            page_size,
        })
    }

    pub fn like_album(
        &self,
        album_id: &str,
        title: &str,
        artist: &str,
        cover: Option<&str>,
    ) -> Result<(), String> {
        let store = self.store.as_ref().ok_or("Database not initialized")?;
        store
            .like_album(album_id, title, artist, cover)
            .map_err(|e| e.to_string())
    }

    pub fn unlike_album(&self, album_id: &str) -> Result<(), String> {
        let store = self.store.as_ref().ok_or("Database not initialized")?;
        store.unlike_album(album_id).map_err(|e| e.to_string())
    }

    pub fn is_album_liked(&self, album_id: &str) -> bool {
        let store = match self.store.as_ref() {
            Some(s) => s,
            None => return false,
        };
        store.is_album_liked(album_id).unwrap_or(false)
    }

    pub fn get_liked_albums(&self) -> Result<Vec<LikedAlbum>, String> {
        let store = self.store.as_ref().ok_or("Database not initialized")?;
        store.get_liked_albums().map_err(|e| e.to_string())
    }

    pub fn like_artist(&self, artist_name: &str) -> Result<(), String> {
        let store = self.store.as_ref().ok_or("Database not initialized")?;
        store.like_artist(artist_name).map_err(|e| e.to_string())
    }

    pub fn unlike_artist(&self, artist_name: &str) -> Result<(), String> {
        let store = self.store.as_ref().ok_or("Database not initialized")?;
        store.unlike_artist(artist_name).map_err(|e| e.to_string())
    }

    pub fn is_artist_liked(&self, artist_name: &str) -> bool {
        let store = match self.store.as_ref() {
            Some(s) => s,
            None => return false,
        };
        store.is_artist_liked(artist_name).unwrap_or(false)
    }

    pub fn get_liked_artists(&self) -> Result<Vec<Artist>, String> {
        let store = self.store.as_ref().ok_or("Database not initialized")?;
        store.get_liked_artists().map_err(|e| e.to_string())
    }

    pub fn add_play_history(&self, track_id: &str) -> Result<(), String> {
        let store = self.store.as_ref().ok_or("Database not initialized")?;
        let history = PlayHistory {
            id: Uuid::new_v4().to_string(),
            track_id: track_id.to_string(),
            played_at: Utc::now().timestamp(),
        };
        store.add_play_history(&history).map_err(|e| e.to_string())
    }

    pub fn get_recently_played(&self, limit: usize) -> Result<Vec<Track>, String> {
        let store = self.store.as_ref().ok_or("Database not initialized")?;
        store.get_recently_played(limit).map_err(|e| e.to_string())
    }

    pub fn clear_play_history(&self) -> Result<(), String> {
        let store = self.store.as_ref().ok_or("Database not initialized")?;
        store.clear_play_history().map_err(|e| e.to_string())
    }

    pub fn get_favorites_summary(&self) -> Result<UserFavorites, String> {
        let store = self.store.as_ref().ok_or("Database not initialized")?;
        Ok(UserFavorites {
            liked_track_ids: store.get_liked_track_ids().map_err(|e| e.to_string())?,
            liked_album_ids: store.get_liked_album_ids().map_err(|e| e.to_string())?,
            liked_artist_names: store
                .get_liked_artists()
                .map_err(|e| e.to_string())?
                .into_iter()
                .map(|a| a.name)
                .collect(),
        })
    }
}
