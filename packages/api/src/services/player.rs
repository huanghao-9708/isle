use crate::infrastructure::audio::{AudioEngine, AudioStatus};
use crate::models::{PlayMode, Track};
use rand::Rng;

/// 播放服务
///
/// 封装全部播放业务逻辑，包括播放控制、队列管理、播放模式切换。
/// 通过 AudioEngine 驱动底层音频解码与输出。
pub struct PlayerService {
    audio_engine: AudioEngine,
    queue: Vec<Track>,
    current_index: usize,
    play_mode: PlayMode,
    volume: u8,
}

impl Default for PlayerService {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerService {
    pub fn new() -> Self {
        PlayerService {
            audio_engine: AudioEngine::new(),
            queue: vec![],
            current_index: 0,
            play_mode: PlayMode::Sequence,
            volume: 70,
        }
    }

    /// 设置新的播放队列
    pub fn set_queue(&mut self, tracks: Vec<Track>) {
        self.queue = tracks;
        self.current_index = 0;
    }

    /// 播放队列中指定索引的音轨
    pub fn play_index(&mut self, index: usize) -> Result<(), String> {
        if let Some(track) = self.queue.get(index) {
            self.current_index = index;
            self.audio_engine.load(&track.path)?;
            self.audio_engine.play()
        } else {
            Err("Index out of bounds".to_string())
        }
    }

    /// 播放指定音轨（自动在队列中定位）
    pub fn play_track(&mut self, track: &Track) -> Result<(), String> {
        if let Some(index) = self.queue.iter().position(|t| t.id == track.id) {
            self.play_index(index)
        } else {
            // 如果不在当前队列，则添加到队列末尾并播放
            self.queue.push(track.clone());
            self.play_index(self.queue.len() - 1)
        }
    }

    pub fn play(&mut self) -> Result<(), String> {
        self.audio_engine.play()
    }

    pub fn pause(&mut self) -> Result<(), String> {
        self.audio_engine.pause()
    }

    pub fn stop(&mut self) -> Result<(), String> {
        self.audio_engine.stop()
    }

    pub fn next(&mut self) -> Result<(), String> {
        if self.queue.is_empty() {
            return Err("Queue is empty".to_string());
        }

        if let Some(next) = self.calculate_next_index() {
            self.play_index(next)
        } else {
            self.stop()
        }
    }

    pub fn prev(&mut self) -> Result<(), String> {
        if self.queue.is_empty() {
            return Err("Queue is empty".to_string());
        }

        let prev = if self.current_index == 0 {
            self.queue.len() - 1
        } else {
            self.current_index - 1
        };
        self.play_index(prev)
    }

    pub fn seek(&mut self, position: u32) -> Result<(), String> {
        self.audio_engine.seek(position)
    }

    pub fn set_volume(&mut self, volume: u8) -> Result<(), String> {
        self.volume = volume.clamp(0, 100);
        self.audio_engine.set_volume(self.volume)
    }

    pub fn set_play_mode(&mut self, mode: PlayMode) {
        self.play_mode = mode;
    }

    /// 定时检测状态，处理自动切歌
    pub fn tick(&mut self) -> Result<bool, String> {
        let status = self.audio_engine.status();
        if status.finished {
            match self.next() {
                Ok(_) => Ok(true),
                Err(_) => Ok(false),
            }
        } else {
            Ok(false)
        }
    }

    pub fn audio_status(&self) -> AudioStatus {
        self.audio_engine.status()
    }

    pub fn current_track(&self) -> Option<&Track> {
        self.queue.get(self.current_index)
    }

    pub fn is_playing(&self) -> bool {
        self.audio_engine.status().is_playing
    }

    pub fn volume(&self) -> u8 {
        self.volume
    }

    pub fn play_mode(&self) -> &PlayMode {
        &self.play_mode
    }

    pub fn current_index(&self) -> usize {
        self.current_index
    }

    pub fn queue(&self) -> &[Track] {
        &self.queue
    }

    // 队列管理方法
    pub fn append_to_queue(&mut self, tracks: Vec<Track>) {
        self.queue.extend(tracks);
    }

    pub fn insert_next(&mut self, track: Track) {
        if self.queue.is_empty() {
            self.queue.push(track);
        } else {
            self.queue.insert(self.current_index + 1, track);
        }
    }

    pub fn remove_from_queue(&mut self, track_id: &str) {
        if let Some(pos) = self.queue.iter().position(|t| t.id == track_id) {
            self.queue.remove(pos);
            if pos <= self.current_index && self.current_index > 0 {
                self.current_index -= 1;
            }
        }
    }

    pub fn clear_queue(&mut self) -> Result<(), String> {
        self.queue.clear();
        self.current_index = 0;
        self.stop()
    }

    fn calculate_next_index(&self) -> Option<usize> {
        let len = self.queue.len();
        if len == 0 {
            return None;
        }

        match self.play_mode {
            PlayMode::Sequence => {
                if self.current_index + 1 < len {
                    Some(self.current_index + 1)
                } else {
                    None
                }
            }
            PlayMode::LoopOne => Some(self.current_index),
            PlayMode::LoopAll => Some((self.current_index + 1) % len),
            PlayMode::Shuffle => {
                let mut rng = rand::thread_rng();
                Some(rng.gen_range(0..len))
            }
        }
    }
}
