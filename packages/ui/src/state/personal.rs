use crate::components::types::Track as UITrack;
use api::models::{Artist, LikedAlbum, PaginatedResult, UserPlaylist as ApiPlaylist};
use api::services::PersonalService;
use dioxus::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Copy, PartialEq)]
pub struct PersonalProvider {
    service: Signal<Arc<Mutex<PersonalService>>>,
    liked_tracks: Signal<Vec<UITrack>>,
    pub playlists: Signal<Vec<ApiPlaylist>>,
    liked_albums: Signal<Vec<LikedAlbum>>,
    liked_artists: Signal<Vec<Artist>>,
    recently_played: Signal<Vec<UITrack>>,
    is_loading: Signal<bool>,
}

impl PersonalProvider {
    pub fn new(service: PersonalService) -> Self {
        Self {
            service: Signal::new(Arc::new(Mutex::new(service))),
            liked_tracks: Signal::new(Vec::new()),
            playlists: Signal::new(Vec::new()),
            liked_albums: Signal::new(Vec::new()),
            liked_artists: Signal::new(Vec::new()),
            recently_played: Signal::new(Vec::new()),
            is_loading: Signal::new(false),
        }
    }

    pub fn init(service_factory: impl FnOnce() -> PersonalService) -> Self {
        let provider = use_hook(|| Self::new(service_factory()));
        use_context_provider(|| provider.clone());
        provider
    }

    pub async fn load_all(&mut self) {
        self.is_loading.set(true);

        let service = self.service.read().clone();
        let svc = service.lock().await;

        if let Ok(tracks) = svc.get_liked_tracks() {
            self.liked_tracks
                .set(tracks.into_iter().map(UITrack::from).collect());
        }

        if let Ok(playlists) = svc.get_all_playlists() {
            self.playlists.set(playlists);
        }

        if let Ok(albums) = svc.get_liked_albums() {
            self.liked_albums.set(albums);
        }

        if let Ok(artists) = svc.get_liked_artists() {
            self.liked_artists.set(artists);
        }

        if let Ok(recent) = svc.get_recently_played(50) {
            self.recently_played
                .set(recent.into_iter().map(UITrack::from).collect());
        }

        self.is_loading.set(false);
    }

    pub fn liked_tracks(&self) -> Vec<UITrack> {
        (self.liked_tracks)()
    }

    pub fn playlists(&self) -> Vec<ApiPlaylist> {
        (self.playlists)()
    }

    pub fn liked_albums(&self) -> Vec<LikedAlbum> {
        (self.liked_albums)()
    }

    pub fn liked_artists(&self) -> Vec<Artist> {
        (self.liked_artists)()
    }

    pub fn recently_played(&self) -> Vec<UITrack> {
        (self.recently_played)()
    }

    pub fn is_loading(&self) -> bool {
        (self.is_loading)()
    }

    pub async fn like_track(&mut self, track_id: &str) -> Result<(), String> {
        let service = self.service.read().clone();
        let svc = service.lock().await;
        svc.like_track(track_id)?;
        drop(svc);
        self.refresh_liked_tracks().await;
        Ok(())
    }

    pub async fn unlike_track(&mut self, track_id: &str) -> Result<(), String> {
        let service = self.service.read().clone();
        let svc = service.lock().await;
        svc.unlike_track(track_id)?;
        drop(svc);
        self.refresh_liked_tracks().await;
        Ok(())
    }

    pub async fn is_track_liked(&self, track_id: &str) -> bool {
        let service = self.service.read().clone();
        let svc = service.lock().await;
        svc.is_track_liked(track_id)
    }

    async fn refresh_liked_tracks(&mut self) {
        let service = self.service.read().clone();
        let svc = service.lock().await;
        if let Ok(tracks) = svc.get_liked_tracks() {
            self.liked_tracks
                .set(tracks.into_iter().map(UITrack::from).collect());
        }
    }

    pub async fn create_playlist(
        &mut self,
        name: &str,
        description: &str,
        tags: Vec<String>,
    ) -> Result<ApiPlaylist, String> {
        let service = self.service.read().clone();
        let svc = service.lock().await;
        let playlist = svc.create_playlist(name, description, tags)?;
        drop(svc);
        self.refresh_playlists().await;
        Ok(playlist)
    }

    pub async fn update_playlist(
        &mut self,
        id: &str,
        name: &str,
        description: &str,
        tags: Vec<String>,
    ) -> Result<(), String> {
        let service = self.service.read().clone();
        let svc = service.lock().await;
        svc.update_playlist(id, name, description, tags)?;
        drop(svc);
        self.refresh_playlists().await;
        Ok(())
    }

    pub async fn delete_playlist(&mut self, id: &str) -> Result<(), String> {
        let service = self.service.read().clone();
        let svc = service.lock().await;
        svc.delete_playlist(id)?;
        drop(svc);
        self.refresh_playlists().await;
        Ok(())
    }

    pub async fn add_track_to_playlist(
        &mut self,
        playlist_id: &str,
        track_id: &str,
    ) -> Result<(), String> {
        let service = self.service.read().clone();
        let svc = service.lock().await;
        svc.add_track_to_playlist(playlist_id, track_id)?;
        drop(svc);
        self.refresh_playlists().await;
        Ok(())
    }

    pub async fn add_track_to_playlist_batch(
        &mut self,
        playlist_id: &str,
        track_ids: Vec<String>,
    ) -> Result<(), String> {
        if track_ids.is_empty() {
            return Ok(());
        }
        let service = self.service.read().clone();
        let svc = service.lock().await;
        svc.add_tracks_to_playlist_batch(playlist_id, &track_ids)?;
        drop(svc);
        self.refresh_playlists().await;
        Ok(())
    }

    pub async fn get_playlist_tracks(&self, playlist_id: &str) -> Result<Vec<UITrack>, String> {
        let res = self
            .get_playlist_tracks_paginated(playlist_id, 10000, 0)
            .await?;
        Ok(res.items.into_iter().map(UITrack::from).collect())
    }

    pub async fn get_liked_tracks_paginated(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<PaginatedResult<api::models::Track>, String> {
        let service = self.service.read().clone();
        let svc = service.lock().await;
        svc.get_liked_tracks_paginated(limit, offset)
    }

    pub async fn get_playlist_tracks_paginated(
        &self,
        playlist_id: &str,
        limit: usize,
        offset: usize,
    ) -> Result<PaginatedResult<api::models::Track>, String> {
        let service = self.service.read().clone();
        let svc = service.lock().await;
        svc.get_playlist_tracks_paginated(playlist_id, limit, offset)
    }

    async fn refresh_playlists(&mut self) {
        let service = self.service.read().clone();
        let svc = service.lock().await;
        if let Ok(playlists) = svc.get_all_playlists() {
            self.playlists.set(playlists);
        }
    }

    pub async fn like_album(
        &mut self,
        album_id: &str,
        title: &str,
        artist: &str,
        cover: Option<&str>,
    ) -> Result<(), String> {
        let service = self.service.read().clone();
        let svc = service.lock().await;
        svc.like_album(album_id, title, artist, cover)?;
        drop(svc);
        self.refresh_liked_albums().await;
        Ok(())
    }

    pub async fn unlike_album(&mut self, album_id: &str) -> Result<(), String> {
        let service = self.service.read().clone();
        let svc = service.lock().await;
        svc.unlike_album(album_id)?;
        drop(svc);
        self.refresh_liked_albums().await;
        Ok(())
    }

    async fn refresh_liked_albums(&mut self) {
        let service = self.service.read().clone();
        let svc = service.lock().await;
        if let Ok(albums) = svc.get_liked_albums() {
            self.liked_albums.set(albums);
        }
    }

    pub async fn like_artist(&mut self, artist_name: &str) -> Result<(), String> {
        let service = self.service.read().clone();
        let svc = service.lock().await;
        svc.like_artist(artist_name)?;
        drop(svc);
        self.refresh_liked_artists().await;
        Ok(())
    }

    pub async fn unlike_artist(&mut self, artist_name: &str) -> Result<(), String> {
        let service = self.service.read().clone();
        let svc = service.lock().await;
        svc.unlike_artist(artist_name)?;
        drop(svc);
        self.refresh_liked_artists().await;
        Ok(())
    }

    async fn refresh_liked_artists(&mut self) {
        let service = self.service.read().clone();
        let svc = service.lock().await;
        if let Ok(artists) = svc.get_liked_artists() {
            self.liked_artists.set(artists);
        }
    }

    pub async fn add_play_history(&mut self, track_id: &str) -> Result<(), String> {
        let service = self.service.read().clone();
        let svc = service.lock().await;
        svc.add_play_history(track_id)?;
        drop(svc);
        self.refresh_recently_played().await;
        Ok(())
    }

    async fn refresh_recently_played(&mut self) {
        let service = self.service.read().clone();
        let svc = service.lock().await;
        if let Ok(recent) = svc.get_recently_played(50) {
            self.recently_played
                .set(recent.into_iter().map(UITrack::from).collect());
        }
    }
}
