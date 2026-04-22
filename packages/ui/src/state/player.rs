use crate::components::types::Track as UITrack;
use api::models::{PlayMode, Track as ApiTrack};
use api::services::PlayerService;
use dioxus::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;

/// 播放引擎全局状态提供者
///
/// 该结构体利用 Dioxus 的 Signal 机制包装了底层的 PlayerService，
/// 使得 UI 组件可以直接消费响应式的播放数据流。
#[derive(Clone, Copy, PartialEq)]
pub struct PlayerProvider {
    /// 底层 PlayerService，使用 Arc<Mutex> 保证跨异步上下文的安全访问
    service: Signal<Arc<Mutex<PlayerService>>>,
    /// 当前正在播放的音轨（UI 格式）
    pub current_track: Signal<Option<UITrack>>,
    /// 是否正在播放
    pub is_playing: Signal<bool>,
    /// 当前播放进度（秒）
    pub progress: Signal<u32>,
    /// 当前音轨总时长（秒）
    pub duration: Signal<u32>,
    /// 当前音量 (0~100)
    pub volume: Signal<u8>,
    /// 当前播放模式
    pub play_mode: Signal<PlayMode>,
    /// 当前播放队列（UI 格式）
    pub queue: Signal<Vec<UITrack>>,
    /// 当前播放索引
    pub queue_index: Signal<usize>,
    /// 播放列表侧边栏是否显示
    pub is_playlist_visible: Signal<bool>,
    /// 沉浸式全屏播放是否显示
    pub is_immersive_visible: Signal<bool>,
}

impl PlayerProvider {
    /// 创建新的 PlayerProvider（非钩子版本）
    pub fn new(service: PlayerService) -> Self {
        Self {
            service: Signal::new(Arc::new(Mutex::new(service))),
            current_track: Signal::new(None),
            is_playing: Signal::new(false),
            progress: Signal::new(0),
            duration: Signal::new(0),
            volume: Signal::new(70),
            play_mode: Signal::new(PlayMode::Sequence),
            queue: Signal::new(Vec::new()),
            queue_index: Signal::new(0),
            is_playlist_visible: Signal::new(false),
            is_immersive_visible: Signal::new(false),
        }
    }

    /// 初始化并注入到 Dioxus Context 中
    pub fn init(service_factory: impl FnOnce() -> PlayerService) -> Self {
        let provider = use_hook(|| Self::new(service_factory()));
        use_context_provider(|| provider);

        // 启动定时轮询协程，定期从 AudioEngine 同步状态到 Signal
        let mut provider_clone = provider;
        use_resource(move || async move {
            provider_clone.start_status_polling().await;
        });

        provider
    }

    /// 定时轮询 AudioEngine 状态并更新 Signal
    async fn start_status_polling(&mut self) {
        loop {
            self.sync_status().await;
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    }

    /// 单次同步底层状态到 Signal
    async fn sync_status(&mut self) {
        let service = self.service.read().clone();
        let mut svc = service.lock().await;

        let status = svc.audio_status();
        self.progress.set(status.position);
        self.duration.set(status.duration);
        self.is_playing.set(status.is_playing);

        // 检测自动切歌
        if let Ok(switched) = svc.tick() {
            if switched {
                self.refresh_current_track(&svc);
            }
        }
    }

    /// 从 Service 刷新当前音轨信息到 Signal
    fn refresh_current_track(&mut self, svc: &PlayerService) {
        if let Some(track) = svc.current_track() {
            self.current_track.set(Some(UITrack::from(track.clone())));
            self.queue_index.set(svc.current_index());
        } else {
            self.current_track.set(None);
        }

        self.queue.set(
            svc.queue()
                .iter()
                .map(|t| UITrack::from(t.clone()))
                .collect(),
        );
    }

    // ──────────── 状态访问便捷方法 ────────────

    pub fn is_playing(&self) -> bool {
        (self.is_playing)()
    }

    pub fn progress(&self) -> u32 {
        (self.progress)()
    }

    pub fn duration(&self) -> u32 {
        (self.duration)()
    }

    pub fn volume(&self) -> u8 {
        (self.volume)()
    }

    pub fn play_mode(&self) -> PlayMode {
        (self.play_mode)()
    }

    // ──────────── 对外暴露的播放控制方法 ────────────

    /// 切换播放/暂停状态
    pub async fn toggle_play(&mut self) {
        let service = self.service.read().clone();
        let mut svc = service.lock().await;
        if svc.is_playing() {
            let _ = svc.pause();
        } else {
            let _ = svc.play();
        }
        self.is_playing.set(svc.is_playing());
    }

    /// 播放队列中指定索引的音轨
    pub async fn play_index(&mut self, index: usize) {
        let service = self.service.read().clone();
        let mut svc = service.lock().await;
        if svc.play_index(index).is_ok() {
            self.refresh_current_track(&svc);
            self.is_playing.set(true);
        }
    }

    /// 播放下一首
    pub async fn next(&mut self) {
        let service = self.service.read().clone();
        let mut svc = service.lock().await;
        if svc.next().is_ok() {
            self.refresh_current_track(&svc);
        }
    }

    /// 播放上一首
    pub async fn prev(&mut self) {
        let service = self.service.read().clone();
        let mut svc = service.lock().await;
        if svc.prev().is_ok() {
            self.refresh_current_track(&svc);
        }
    }

    /// 跳转到指定秒数位置
    pub async fn seek(&mut self, position: u32) {
        let service = self.service.read().clone();
        let mut svc = service.lock().await;
        let _ = svc.seek(position);
        self.progress.set(position);
    }

    /// 设置音量
    pub async fn set_volume(&mut self, volume: u8) {
        let service = self.service.read().clone();
        let mut svc = service.lock().await;
        let _ = svc.set_volume(volume);
        self.volume.set(volume);
    }

    /// 切换播放模式
    pub async fn toggle_play_mode(&mut self) {
        let service = self.service.read().clone();
        let mut svc = service.lock().await;
        let new_mode = match svc.play_mode() {
            PlayMode::Sequence => PlayMode::LoopAll,
            PlayMode::LoopAll => PlayMode::LoopOne,
            PlayMode::LoopOne => PlayMode::Shuffle,
            PlayMode::Shuffle => PlayMode::Sequence,
        };
        svc.set_play_mode(new_mode.clone());
        self.play_mode.set(new_mode);
    }

    /// 设置新的播放队列并开始播放指定索引的音轨
    pub async fn play_from_list(&mut self, tracks: Vec<ApiTrack>, index: usize) {
        let service = self.service.read().clone();
        let mut svc = service.lock().await;
        svc.set_queue(tracks.clone());
        let _ = svc.play_index(index);
        self.refresh_current_track(&svc);
        self.is_playing.set(true);
    }

    /// 设置新的播放队列并开始播放第一首
    pub async fn set_queue_and_play(&mut self, tracks: Vec<ApiTrack>) {
        self.play_from_list(tracks, 0).await;
    }

    /// 从队列中删除指定音轨
    pub async fn remove_from_queue(&mut self, track_id: String) {
        let service = self.service.read().clone();
        let mut svc = service.lock().await;
        svc.remove_from_queue(&track_id);
        self.refresh_current_track(&svc);
    }

    /// 清空播放队列
    pub async fn clear_queue(&mut self) {
        let service = self.service.read().clone();
        let mut svc = service.lock().await;
        let _ = svc.clear_queue();
        self.refresh_current_track(&svc);
        self.is_playing.set(false);
    }
}
