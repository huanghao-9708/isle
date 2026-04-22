use crate::models::Track;
use std::collections::HashMap;

pub struct Cache {
    track_cache: HashMap<String, Track>,
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

impl Cache {
    pub fn new() -> Self {
        Cache {
            track_cache: HashMap::new(),
        }
    }

    pub fn get_track(&self, id: &str) -> Option<&Track> {
        self.track_cache.get(id)
    }

    pub fn set_track(&mut self, track: Track) {
        self.track_cache.insert(track.id.clone(), track);
    }

    pub fn remove_track(&mut self, id: &str) {
        self.track_cache.remove(id);
    }

    /// 根据路径前缀批量移除缓存中的音轨
    pub fn remove_tracks_by_path_prefix(&mut self, prefix: &str) {
        self.track_cache
            .retain(|_, track| !track.path.starts_with(prefix));
    }

    pub fn clear(&mut self) {
        self.track_cache.clear();
    }
}
