# 「私屿」播放引擎模块实施方案 (Player Engine Implementation)

**文档版本**：V1.0
**修订日期**：2026年04月
**状态**：方案设计阶段
**关联文档**：@project.md (需求锚定), @technical_solution.md (技术方案), @music_library_implementation.md (音乐库实施方案)

---

## 一、 功能概述

播放引擎模块是「私屿」的播放核心，负责音频文件的解码、输出、播放控制及播放队列管理。本方案设计包含三个核心层次：

1. **基础设施层** (`api/src/infrastructure/audio/`)：底层音频解码与输出引擎
2. **业务逻辑层** (`api/src/services/player.rs`)：播放服务，封装播放控制、队列管理、播放模式等业务逻辑
3. **UI 适配层** (`ui/src/state/player.rs`)：PlayerProvider，参照 `LibraryProvider` 模式，桥接 Service 与 Dioxus 响应式 UI

---

## 二、 核心功能需求

### 2.1 播放控制 (Playback Control)

1. **基础播放操作**：
    * **播放/暂停**：切换当前音轨的播放/暂停状态
    * **停止**：停止播放并重置进度至起点
    * **上一首/下一首**：按当前播放模式切换音轨
    * **进度跳转 (Seek)**：精确跳转到指定时间点

2. **音量控制**：
    * 支持 0~100 级别音量调节
    * 实时生效，无需重新加载音轨

### 2.2 播放队列管理 (Queue Management)

1. **队列操作**：
    * **设置队列**：用外部传入的音轨列表替换当前播放队列
    * **追加队列**：在队列末尾追加音轨
    * **插入下一首**：在当前播放位置之后插入音轨
    * **移除**：从队列中移除指定音轨
    * **清空队列**：清空并停止播放

2. **播放模式**：
    * **顺序播放** (Sequence)：按队列顺序依次播放
    * **单曲循环** (LoopOne)：重复当前音轨
    * **列表循环** (LoopAll)：队列播完后从头开始
    * **随机播放** (Shuffle)：随机选取队列中的音轨

### 2.3 播放状态 (Playback State)

UI 层需要实时消费以下状态数据：

| 状态字段 | 类型 | 说明 |
|---|---|---|
| `current_track` | `Option<Track>` | 当前正在播放的音轨信息 |
| `is_playing` | `bool` | 是否正在播放 |
| `progress` | `u32` | 当前播放进度（秒） |
| `duration` | `u32` | 当前音轨总时长（秒） |
| `volume` | `u8` | 当前音量（0~100） |
| `play_mode` | `PlayMode` | 当前播放模式 |
| `queue` | `Vec<Track>` | 当前播放队列 |
| `queue_index` | `usize` | 当前播放索引 |

---

## 三、 技术实现设计

### 3.1 模块依赖关系

```
┌────────────────────────────────────────────────────────┐
│                    UI 层 (ui 包)                        │
│  ┌──────────────────────────────────────────────────┐  │
│  │  PlayerProvider (ui/src/state/player.rs)          │  │
│  │  - 持有 PlayerService (Arc<Mutex>)               │  │
│  │  - 暴露 Signal 给 UI 组件                         │  │
│  │  - 提供 play/pause/next/prev/seek 等异步方法      │  │
│  └──────────────────────────────────────────────────┘  │
│                         │                               │
│                         ▼                               │
├────────────────────────────────────────────────────────┤
│                业务逻辑层 (api 包 services/)            │
│  ┌──────────────────────────────────────────────────┐  │
│  │  PlayerService (api/src/services/player.rs)      │  │
│  │  - 封装播放逻辑、队列管理、模式切换               │  │
│  │  - 调用 AudioEngine 实现底层播放控制               │  │
│  └──────────────────────────────────────────────────┘  │
│                         │                               │
│                         ▼                               │
├────────────────────────────────────────────────────────┤
│              基础设施层 (api 包 infrastructure/)        │
│  ┌──────────────────────────────────────────────────┐  │
│  │  AudioEngine (api/src/infrastructure/audio/)     │  │
│  │  - 音频解码 (Symphonia)                           │  │
│  │  - 音频输出 (Rodio)                               │  │
│  │  - 独立音频线程                                    │  │
│  └──────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────┘
```

### 3.2 基础设施层 — AudioEngine 重构

**文件**：`packages/api/src/infrastructure/audio/mod.rs`

当前 `AudioEngine` 是空壳实现（所有方法返回 `Ok(())`），需要重构为具备真实音频能力的引擎。

#### 3.2.1 核心结构

```rust
use std::sync::{Arc, Mutex};
use std::sync::mpsc;

/// 音频引擎命令枚举
///
/// 用于从业务层向音频播放线程发送控制指令。
pub enum AudioCommand {
    /// 加载指定路径的音频文件并开始播放
    /// 参数: 音频文件的物理路径
    Load(String),
    /// 恢复当前暂停的播放
    Play,
    /// 暂停当前播放
    Pause,
    /// 停止播放并释放资源
    Stop,
    /// 跳转到指定秒数位置
    /// 参数: 目标秒数
    Seek(u32),
    /// 设置音量
    /// 参数: 音量值 (0~100)
    SetVolume(u8),
}

/// 音频引擎状态快照
///
/// 从音频线程同步回来的只读状态，供 Service 层读取。
#[derive(Clone, Debug)]
pub struct AudioStatus {
    /// 当前播放进度（秒）
    pub position: u32,
    /// 当前音轨总时长（秒）
    pub duration: u32,
    /// 是否正在播放
    pub is_playing: bool,
    /// 是否已到达音轨末尾（用于触发自动切歌）
    pub finished: bool,
}

/// 音频引擎
///
/// 管理独立的音频播放线程，通过 mpsc channel 发送命令，
/// 通过 Arc<Mutex<AudioStatus>> 共享状态。
///
/// # 示例
/// ```rust
/// let engine = AudioEngine::new();
/// engine.load("D:/Music/song.mp3")?;
/// engine.play()?;
/// engine.set_volume(80)?;
/// let status = engine.status();
/// ```
pub struct AudioEngine {
    /// 向音频线程发送控制命令的通道
    command_tx: mpsc::Sender<AudioCommand>,
    /// 从音频线程读取的共享状态
    status: Arc<Mutex<AudioStatus>>,
}
```

#### 3.2.2 关键接口设计

```rust
impl AudioEngine {
    /// 创建新的音频引擎实例，启动独立的音频播放线程。
    ///
    /// # 示例
    /// ```rust
    /// let engine = AudioEngine::new();
    /// ```
    pub fn new() -> Self { /* ... */ }

    /// 加载指定路径的音频文件。
    ///
    /// # 参数
    /// * `path` - 音频文件的物理路径，例如 "D:/Music/song.flac"
    ///
    /// # 示例
    /// ```rust
    /// engine.load("D:/Music/周杰伦 - 七里香.flac")?;
    /// ```
    pub fn load(&self, path: &str) -> Result<(), String> { /* ... */ }

    /// 恢复播放当前已加载的音频。
    pub fn play(&self) -> Result<(), String> { /* ... */ }

    /// 暂停播放。
    pub fn pause(&self) -> Result<(), String> { /* ... */ }

    /// 停止播放并重置进度。
    pub fn stop(&self) -> Result<(), String> { /* ... */ }

    /// 跳转到指定的秒数位置。
    ///
    /// # 参数
    /// * `position` - 目标秒数，例如传入 120 表示跳转到 2:00
    pub fn seek(&self, position: u32) -> Result<(), String> { /* ... */ }

    /// 设置音量。
    ///
    /// # 参数
    /// * `volume` - 音量值 (0~100)，超出范围将被 clamp
    pub fn set_volume(&self, volume: u8) -> Result<(), String> { /* ... */ }

    /// 获取当前音频播放状态快照。
    ///
    /// # 示例
    /// ```rust
    /// let status = engine.status();
    /// println!("进度: {}/{}秒", status.position, status.duration);
    /// ```
    pub fn status(&self) -> AudioStatus { /* ... */ }
}
```

#### 3.2.3 音频线程设计

```
主线程 (UI/Service)          音频线程
     │                          │
     │── Load(path) ──────────► │  解码文件 → 建立输出流
     │── Play ──────────────► │  恢复播放
     │── Pause ─────────────► │  暂停输出
     │── Seek(pos) ─────────► │  重定位解码器
     │── SetVolume(vol) ────► │  调整增益
     │                          │
     │◄── status (共享内存) ──── │  每 100ms 更新进度
```

#### 3.2.4 MVP 阶段实现策略

> **重要**：MVP 阶段先实现完整的接口骨架和状态流转，底层解码可使用 mock 逻辑（模拟进度递增），后续接入 Symphonia + Rodio 真实解码。这样可以先跑通整个播放流程再深入底层。

### 3.3 业务逻辑层 — PlayerService 重构

**文件**：`packages/api/src/services/player.rs`

当前 `PlayerService` 的问题：
1. 缺少 `load` 操作（播放指定音轨时需先加载文件）
2. 缺少 `seek` 方法
3. `next()` 的随机播放逻辑未真正实现
4. 缺少自动切歌（播完一首自动跳下一首）的检测能力
5. 缺少 `duration`、`progress` 的实时查询能力

#### 3.3.1 重构后的核心结构

```rust
use crate::models::{Track, PlayMode};
use crate::infrastructure::audio::{AudioEngine, AudioStatus};
use rand::Rng;

/// 播放服务
///
/// 封装全部播放业务逻辑，包括播放控制、队列管理、播放模式切换。
/// 通过 AudioEngine 驱动底层音频解码与输出。
///
/// # 示例
/// ```rust
/// let mut player = PlayerService::new();
/// player.set_queue(tracks);
/// player.play_index(0)?;  // 播放队列第一首
/// player.set_volume(80)?; // 设置音量
/// ```
pub struct PlayerService {
    /// 底层音频引擎
    audio_engine: AudioEngine,
    /// 播放队列
    queue: Vec<Track>,
    /// 当前播放索引
    current_index: usize,
    /// 播放模式
    play_mode: PlayMode,
    /// 音量 (0~100)
    volume: u8,
}
```

#### 3.3.2 新增/重构的核心方法

```rust
impl PlayerService {
    /// 播放队列中指定索引的音轨。
    ///
    /// # 参数
    /// * `index` - 队列中的索引位置
    ///
    /// # 示例
    /// ```rust
    /// player.set_queue(tracks);
    /// player.play_index(3)?; // 播放第4首
    /// ```
    pub fn play_index(&mut self, index: usize) -> Result<(), String> { /* ... */ }

    /// 播放指定 Track（自动在队列中定位）。
    ///
    /// # 参数
    /// * `track` - 要播放的音轨对象
    ///
    /// # 示例
    /// ```rust
    /// player.play_track(my_track)?;
    /// ```
    pub fn play_track(&mut self, track: &Track) -> Result<(), String> { /* ... */ }

    /// 跳转到指定秒数位置。
    ///
    /// # 参数
    /// * `position` - 目标秒数
    pub fn seek(&mut self, position: u32) -> Result<(), String> { /* ... */ }

    /// 获取当前音频状态快照（进度、时长、播放状态）。
    pub fn audio_status(&self) -> AudioStatus { /* ... */ }

    /// 检测当前音轨是否播放完毕，如果是则自动切到下一首。
    /// 此方法应由 UI 层的定时轮询调用。
    ///
    /// # 返回值
    /// 如果发生了自动切歌，返回 `true`
    pub fn tick(&mut self) -> Result<bool, String> { /* ... */ }

    /// 向队列末尾追加音轨。
    ///
    /// # 参数
    /// * `tracks` - 要追加的音轨列表
    pub fn append_to_queue(&mut self, tracks: Vec<Track>) { /* ... */ }

    /// 在当前播放位置之后插入一首音轨。
    ///
    /// # 参数
    /// * `track` - 要插入的音轨
    pub fn insert_next(&mut self, track: Track) { /* ... */ }

    /// 从队列中移除指定 ID 的音轨。
    ///
    /// # 参数
    /// * `track_id` - 音轨的唯一标识
    pub fn remove_from_queue(&mut self, track_id: &str) { /* ... */ }

    /// 清空播放队列并停止播放。
    pub fn clear_queue(&mut self) -> Result<(), String> { /* ... */ }

    /// 获取当前播放队列。
    pub fn queue(&self) -> &[Track] { /* ... */ }

    /// 获取当前播放索引。
    pub fn current_index(&self) -> usize { /* ... */ }

    /// 获取当前播放模式。
    pub fn play_mode(&self) -> &PlayMode { /* ... */ }
}
```

#### 3.3.3 切歌逻辑详细设计

```rust
/// 根据当前播放模式计算下一首的索引。
///
/// # 参数
/// * `current` - 当前索引
/// * `queue_len` - 队列长度
/// * `mode` - 播放模式
///
/// # 返回值
/// `Some(index)` 为下一首索引；`None` 表示队列已播完（顺序模式末尾）
fn next_index(current: usize, queue_len: usize, mode: &PlayMode) -> Option<usize> {
    match mode {
        PlayMode::Sequence => {
            if current + 1 < queue_len { Some(current + 1) } else { None }
        }
        PlayMode::LoopOne => Some(current),
        PlayMode::LoopAll => Some((current + 1) % queue_len),
        PlayMode::Shuffle => {
            let mut rng = rand::thread_rng();
            Some(rng.gen_range(0..queue_len))
        }
    }
}
```

### 3.4 UI 适配层 — PlayerProvider

**文件**：`packages/ui/src/state/player.rs`

参照已有的 `LibraryProvider` 模式设计，使用 `Signal` 包装状态，通过 `use_context_provider` 注入全局上下文。

#### 3.4.1 核心结构

```rust
use api::models::{Track as ApiTrack, PlayMode};
use api::services::PlayerService;
use crate::components::types::Track as UITrack;
use dioxus::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;

/// 播放引擎全局状态提供者
///
/// 该结构体利用 Dioxus 的 Signal 机制包装了底层的 PlayerService，
/// 使得 UI 组件可以直接消费响应式的播放数据流。
/// 类似 LibraryProvider 的设计模式。
///
/// # 使用方式
///
/// 在 App 根组件中初始化：
/// ```rust
/// let player_provider = PlayerProvider::init(PlayerService::new());
/// ```
///
/// 在子组件中消费：
/// ```rust
/// let player = use_context::<PlayerProvider>();
/// rsx! {
///     div { "当前歌曲: {player.current_track.read().as_ref().map(|t| &t.title).unwrap_or(&\"无\".to_string())}" }
///     button {
///         onclick: move |_| { player.toggle_play(); },
///         if player.is_playing() { "暂停" } else { "播放" }
///     }
/// }
/// ```
#[derive(Clone, Copy)]
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
}
```

#### 3.4.2 初始化与上下文注入

```rust
impl PlayerProvider {
    /// 创建新的 PlayerProvider（非钩子版本）
    ///
    /// # 参数
    /// * `service` - 已构建的 PlayerService 实例
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
        }
    }

    /// 初始化并注入到 Dioxus Context 中。
    ///
    /// 同时启动一个定时轮询协程，每 500ms 同步音频引擎状态到 Signal。
    ///
    /// # 参数
    /// * `service` - PlayerService 实例
    ///
    /// # 示例
    /// ```rust
    /// // 在 App 根组件中调用
    /// let player = PlayerProvider::init(PlayerService::new());
    /// ```
    pub fn init(service: PlayerService) -> Self {
        let provider = Self::new(service);
        use_context_provider(|| provider);

        // 启动定时轮询协程，定期从 AudioEngine 同步状态到 Signal
        let mut provider_clone = provider;
        use_resource(move || async move {
            provider_clone.start_status_polling().await;
        });

        provider
    }
}
```

#### 3.4.3 核心方法

```rust
impl PlayerProvider {
    /// 定时轮询 AudioEngine 状态并更新 Signal。
    ///
    /// 每 500ms 从底层引擎同步一次播放进度、状态等信息，
    /// 同时检测音轨是否播放完毕以触发自动切歌。
    async fn start_status_polling(&mut self) {
        loop {
            self.sync_status().await;
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    }

    /// 单次同步底层状态到 Signal。
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

    /// 从 Service 刷新当前音轨信息到 Signal。
    fn refresh_current_track(&mut self, svc: &PlayerService) {
        if let Some(track) = svc.current_track() {
            self.current_track.set(Some(UITrack::from(track.clone())));
            self.queue_index.set(svc.current_index());
        } else {
            self.current_track.set(None);
        }
    }

    // ──────────── 对外暴露的播放控制方法 ────────────

    /// 切换播放/暂停状态。
    ///
    /// # 示例
    /// ```rust
    /// player.toggle_play().await;
    /// ```
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

    /// 播放队列中指定索引的音轨。
    ///
    /// # 参数
    /// * `index` - 队列中的索引
    pub async fn play_index(&mut self, index: usize) {
        let service = self.service.read().clone();
        let mut svc = service.lock().await;
        if svc.play_index(index).is_ok() {
            self.refresh_current_track(&svc);
            self.is_playing.set(true);
        }
    }

    /// 播放下一首。
    pub async fn next(&mut self) {
        let service = self.service.read().clone();
        let mut svc = service.lock().await;
        if svc.next().is_ok() {
            self.refresh_current_track(&svc);
        }
    }

    /// 播放上一首。
    pub async fn prev(&mut self) {
        let service = self.service.read().clone();
        let mut svc = service.lock().await;
        if svc.prev().is_ok() {
            self.refresh_current_track(&svc);
        }
    }

    /// 跳转到指定秒数位置。
    ///
    /// # 参数
    /// * `position` - 目标秒数
    pub async fn seek(&mut self, position: u32) {
        let service = self.service.read().clone();
        let mut svc = service.lock().await;
        let _ = svc.seek(position);
        self.progress.set(position);
    }

    /// 设置音量。
    ///
    /// # 参数
    /// * `volume` - 音量值 (0~100)
    pub async fn set_volume(&mut self, volume: u8) {
        let service = self.service.read().clone();
        let mut svc = service.lock().await;
        let _ = svc.set_volume(volume);
        self.volume.set(volume);
    }

    /// 切换播放模式（顺序 → 列表循环 → 单曲循环 → 随机 → 顺序）。
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

    /// 设置新的播放队列并开始播放第一首。
    ///
    /// # 参数
    /// * `tracks` - 新的音轨列表（API 格式）
    pub async fn set_queue_and_play(&mut self, tracks: Vec<ApiTrack>) {
        let service = self.service.read().clone();
        let mut svc = service.lock().await;
        svc.set_queue(tracks.clone());
        let _ = svc.play_index(0);
        self.queue.set(tracks.into_iter().map(UITrack::from).collect());
        self.refresh_current_track(&svc);
        self.is_playing.set(true);
    }
}
```

### 3.5 App 根组件集成

在 App 根组件中初始化 PlayerProvider，位置紧随 LibraryProvider 之后：

```rust
#[component]
fn App() -> Element {
    // 1. 初始化音乐库服务
    let library_provider = LibraryProvider::init(LibraryService::new());

    // 2. 初始化播放引擎
    let player_provider = PlayerProvider::init(PlayerService::new());

    rsx! {
        // ... 路由和布局
    }
}
```

在任意子组件中消费：

```rust
#[component]
fn SomeChild() -> Element {
    let player = use_context::<PlayerProvider>();
    // 直接读取 player.is_playing、player.current_track 等 Signal
    rsx! { /* ... */ }
}
```

---

## 四、 需要新增/修改的文件清单

### 4.1 api 包（核心逻辑）

| 操作 | 文件路径 | 说明 |
|---|---|---|
| **重构** | `packages/api/src/infrastructure/audio/mod.rs` | AudioEngine 重构：新增 `AudioCommand`、`AudioStatus`，实现 `load`、`seek`、`status` 等接口，MVP 阶段使用 mock 实现 |
| **重构** | `packages/api/src/services/player.rs` | PlayerService 重构：新增 `play_index`、`play_track`、`seek`、`tick`、`audio_status`、`append_to_queue`、`insert_next`、`remove_from_queue`、`clear_queue` 等方法 |
| **修改** | `packages/api/src/models/playlist.rs` | PlayMode 添加 `Clone` derive（如果尚未有）以便 Signal 使用 |
| **无需修改** | `packages/api/src/lib.rs` | PlayerService 已导出，无需改动 |

### 4.2 ui 包（UI 适配层）

| 操作 | 文件路径 | 说明 |
|---|---|---|
| **新增** | `packages/ui/src/state/player.rs` | PlayerProvider 实现 |
| **修改** | `packages/ui/src/state/mod.rs` | 添加 `pub mod player;` 和 `pub use player::PlayerProvider;` |
| **修改** | `packages/ui/src/components/player/player_bar.rs` | 改造为从 `PlayerProvider` 读取状态，而非 props 传入 |

### 4.3 平台入口（可选）

| 操作 | 文件路径 | 说明 |
|---|---|---|
| **修改** | 各平台 `main.rs` | 在 App 组件中初始化 `PlayerProvider` |

---

## 五、 实施步骤

### 第 1 步：基础设施层 — AudioEngine 重构
1. 定义 `AudioCommand`、`AudioStatus` 数据结构
2. 重构 `AudioEngine`，添加 `load`、`seek`、`status` 接口
3. MVP 阶段内部使用 mock 实现（模拟进度递增，不接入真实解码）
4. 保持现有 `play`、`pause`、`stop`、`set_volume` 接口签名兼容

### 第 2 步：业务逻辑层 — PlayerService 重构
1. 重构现有 `PlayerService`，新增 `play_index`、`seek`、`tick`、`audio_status` 等方法
2. 实现完整的队列管理（`append_to_queue`、`insert_next`、`remove_from_queue`、`clear_queue`）
3. 完善切歌逻辑（`next_index` 函数），确保四种播放模式均正确工作
4. 移除 `is_playing` 字段，改为从 `AudioEngine::status()` 读取

### 第 3 步：UI 适配层 — PlayerProvider
1. 新建 `packages/ui/src/state/player.rs`
2. 实现 `PlayerProvider` 结构体，参照 `LibraryProvider` 模式
3. 在 `packages/ui/src/state/mod.rs` 中导出
4. 实现定时轮询协程和所有对外方法

### 第 4 步：UI 组件对接
1. 改造 `PlayerBar` 组件，从 `use_context::<PlayerProvider>()` 读取状态
2. 绑定所有按钮事件到 `PlayerProvider` 的方法
3. 进度条改为实时读取 `progress` / `duration` Signal

### 第 5 步：集成测试
1. 在 App 根组件中初始化 `PlayerProvider`
2. 验证播放/暂停、切歌、进度更新、音量调节等核心流程
3. 验证四种播放模式的切换逻辑

---

## 六、 隐私与性能准则

1. **100% 本地化**：所有音频解码与播放均在用户本机执行，绝不上传任何音频数据
2. **独立线程**：音频引擎运行在独立线程，不阻塞 UI 渲染
3. **低频轮询**：状态同步频率 500ms，在保证 UI 流畅的同时降低 CPU 开销
4. **按需加载**：仅在用户触发播放时才加载音频文件，不做预加载

---

## 七、 后续迭代规划 (P1)

1. **真实解码接入**：接入 Symphonia + Rodio 替换 mock 实现，支持 FLAC/MP3/WAV/M4A 等格式
2. **歌曲自然结束事件**：从轮询改为事件驱动（基于 Rodio 的 `Sink::sleep_until_end`）
3. **播放历史记录**：记录用户播放历史，支持"最近播放"视图
4. **均衡器预设**：集成基础均衡器能力
5. **跨设备同步**：播放进度通过 WebDAV 同步
