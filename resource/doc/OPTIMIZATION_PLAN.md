# Isle 音乐播放器 — 性能优化方案

> 问题现象：加载 2000+ 首歌时，应用卡顿严重，内存飙升至 ~6 GB。
> 分析日期：2026-04-18

---

## 一、问题根因分析

经过对 `TrackTable`、`LibraryProvider`、数据模型及封面加载链路的完整审计，总结出以下 **4 个核心瓶颈**：

### 1. 全量 DOM 渲染（卡顿主因）

| 项目 | 当前状态 |
|------|----------|
| 文件 | `packages/ui/src/components/music_library/track_table.rs` 第 44 行 |
| 现象 | `tracks.read().iter().enumerate().map(...)` 一次性为全部歌曲创建 DOM 行 |
| 影响 | 2000 首歌 × 每行约 20 个 DOM 节点 ≈ **40,000+ DOM 节点**，WebView2 渲染引擎严重过载 |

### 2. 数据三重克隆（内存主因）

| 项目 | 当前状态 |
|------|----------|
| 文件 | `packages/ui/src/state/library.rs` 第 180-186 行 |
| 现象 | 同一份查询结果被 `clone()` 后分别存入 `all_api_tracks`、`current_api_tracks` 和 `tracks` |
| 影响 | 2000 条记录 × 3 份完整副本，Rust 堆内存至少翻 3 倍 |

```rust
// 当前代码 — 三次克隆
self.all_api_tracks.set(tracks.clone());       // 副本 1
self.current_api_tracks.set(tracks.clone());    // 副本 2
self.tracks.set(tracks.into_iter().map(UITrack::from).collect()); // 副本 3
```

### 3. 歌词全量加载（内存浪费）

| 项目 | 当前状态 |
|------|----------|
| 文件 | `packages/api/src/models/track.rs` 第 18 行 |
| 现象 | `lyrics: Option<String>` 随每首歌从数据库全量读取 |
| 影响 | 歌词在列表页完全不展示，但平均每首歌词 5-10 KB，2000 首 × 3 副本 = **30-60 MB 纯文本浪费** |

### 4. 封面图片全量并发加载（内存 + 网络压力）

| 项目 | 当前状态 |
|------|----------|
| 文件 | `track_table.rs` 第 85 行 / `desktop/src/main.rs` 第 21-58 行 |
| 现象 | 每首歌渲染一个 `<img src="http://covers.localhost/{filename}">"`，2000 首同时发起请求 |
| 影响 | 2000 张图片全部加载到 WebView 内存（约 50KB × 2000 = **~100 MB**），且每次请求触发同步磁盘 I/O |

---

## 二、优化方案

### P0（最高优先级）：重构列表展示策略（首页分页限量 + 虚拟滚动结合）

**目标**：控制 DOM 节点数量并优化性能，同时兼顾用户体验。

**详细策略**：

1. **全局精简列表（所有含有音乐列表的视图）**：
   - 包括**主库歌曲列表、我喜欢的音乐、歌单详情、专辑详情、艺术家详情**等**所有**存在长列表的页面。
   - 默认仅展示最多 **20 条**数据。
   - 在列表最下方右侧统一新增一个 **"查看全部"** 按钮。
   - 避免在任何实体详情页或首页渲染超长列表，全站统一规范体验。

2. **通用详情页高级列表（分页 + 虚拟滚动）**：
   - 点击任何页面中的"查看全部"后，进入统一的高级**全量歌曲详情页组件**。
   - 这个模块采用**服务端分页加载模式**，每页严格限定返回规定数量（如20条）的数据。
   - 结合**虚拟滚动和受限 DOM 渲染**机制，确保不论总量多大，实际渲染的 DOM 永远控制在视图单页范围内，对 WebView2 极其友好。

**实施要点**：

- 页面 UI 层分为“精简首页版本”和“详情页完整版本”。
- 确保服务端（如 `filter_tracks`） API 已支持基于 `LIMIT` 和 `OFFSET` 的分页获取能力，提供接口响应页码及总数量。

**预期效果**：

| 指标 | 优化前 | 优化后 |
|------|--------|--------|
| 首页 DOM 节点数 | ~40,000 | < 400 |
| 首屏加载延迟 | 数秒 | <100ms |
| 内存泄露/飙升风险| 极高 (6GB) | 极低 |

---

### P1（高优先级）：消除数据三重克隆

**目标**：同一份数据只保留一个权威副本。

**方案**：

```rust
// 优化后 — 只保留一份数据
pub async fn refresh_tracks(&mut self) {
    let filter = self.filter.read().clone();
    let service = self.service.read().clone();
    let svc = service.lock().await;

    let result = if filter == TrackFilter::default() {
        svc.get_all_tracks()
    } else {
        svc.filter_tracks(filter, 1, 10000).map(|r| r.items)
    };

    if let Ok(tracks) = result {
        let count = tracks.len();
        // 只保留 api_tracks 作为唯一数据源
        self.current_api_tracks.set(tracks);
        // tracks (UITrack) 通过 Memo 或即时计算生成，不再存储
        info!("LibraryProvider: 音轨列表同步完成, 共 {} 条", count);
    }
}
```

**配合 UI 层改造**：`TrackTable` 直接消费 `Signal<Vec<ApiTrack>>`，在渲染时按需转换为 `UITrack`，避免预转换存储。

**可移除的 Signal**：

- `all_api_tracks` — 完全冗余
- `tracks: Signal<Vec<UITrack>>` — 改为即时计算

---

### P2（中优先级）：歌词延迟加载

**目标**：列表查询不加载歌词，仅在播放/查看歌词时按需获取。

**方案**：

1. **数据库查询层**：`get_all_tracks()` 的 SQL 中排除 `lyrics` 列

```sql
-- 优化前
SELECT * FROM tracks;

-- 优化后
SELECT id, path, title, artist, artist_id, album, album_id,
       duration, size, bitrate, extension, genres, added_at, mtime, cover
FROM tracks;
```

1. **新增按需接口**：

```rust
impl LibraryService {
    /// 按需获取单首歌的歌词
    pub fn get_lyrics(&self, track_id: &str) -> Result<Option<String>, String> {
        // SELECT lyrics FROM tracks WHERE id = ?
    }
}
```

1. **播放器层调用**：在 `PlayerProvider::play()` 时才调用 `get_lyrics()`。

---

### P3（低优先级）：封面图片缓存优化

**目标**：减少重复磁盘 I/O，利用浏览器缓存。

**方案 A：添加 Cache-Control 响应头**

```rust
// desktop/src/main.rs 的 covers 协议处理
Response::builder()
    .header("Content-Type", mime)
    .header("Access-Control-Allow-Origin", "*")
    .header("Cache-Control", "public, max-age=31536000, immutable")  // 新增
    .body(Cow::Owned(data))
    .unwrap()
```

**方案 B：Rust 侧 LRU 内存缓存**（可选）

```rust
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref COVER_CACHE: Mutex<lru::LruCache<String, Vec<u8>>> =
        Mutex::new(lru::LruCache::new(std::num::NonZeroUsize::new(200).unwrap()));
}
```

---

## 三、实施路线图

```
阶段 1（立即）
├── P0: 虚拟滚动列表 ← 解决卡顿 + 大幅降低内存
└── P2: 歌词延迟加载 ← 简单改动，立竿见影减少内存

阶段 2（短期）
├── P1: 消除数据克隆 ← 需要重构状态层
└── P3: 封面缓存优化 ← 一行代码改动
```

## 四、预期优化效果

| 指标 | 优化前（2000 首歌） | 目标 |
|------|---------------------|------|
| 内存占用 | ~6 GB | < 500 MB |
| DOM 节点数 | ~40,000 | < 500 |
| 首屏渲染 | 数秒卡顿 | < 100ms |
| 滚动流畅度 | 严重掉帧 | 60 FPS |
| 封面并发请求 | 2000 | ~20 |
