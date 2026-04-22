use crate::models::Track;
use dioxus::prelude::*;

pub struct LibraryState {
    pub tracks: Signal<Vec<Track>>,
    pub is_loading: Signal<bool>,
    pub scan_progress: Signal<f32>,
    pub search_query: Signal<String>,
}

impl Default for LibraryState {
    fn default() -> Self {
        Self::new()
    }
}

impl LibraryState {
    pub fn new() -> Self {
        LibraryState {
            tracks: use_signal(std::vec::Vec::new),
            is_loading: use_signal(|| false),
            scan_progress: use_signal(|| 0.0),
            search_query: use_signal(String::new),
        }
    }
}
