use crate::models::SyncStatus;
use dioxus::prelude::*;

pub struct SyncState {
    pub status: Signal<SyncStatus>,
    pub progress: Signal<f32>,
    pub last_sync: Signal<Option<u64>>,
}

impl Default for SyncState {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncState {
    pub fn new() -> Self {
        SyncState {
            status: use_signal(|| SyncStatus::Idle),
            progress: use_signal(|| 0.0),
            last_sync: use_signal(|| None),
        }
    }
}
