use crate::models::Track;
use dioxus::prelude::*;

pub struct PlayerState {
    pub current_track: Signal<Option<Track>>,
    pub is_playing: Signal<bool>,
    pub progress: Signal<u32>,
    pub volume: Signal<u8>,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerState {
    pub fn new() -> Self {
        PlayerState {
            current_track: use_signal(|| None),
            is_playing: use_signal(|| false),
            progress: use_signal(|| 0),
            volume: use_signal(|| 70),
        }
    }
}
