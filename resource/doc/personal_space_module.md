# 「私屿」私人空间模块 实现方案

**文档版本**：V1.0
**修订日期**：2026年04月
**文档状态**：初稿
**模块定位**：P0 MVP 核心功能

---

## 一、模块概述

### 1.1 模块背景

「私屿」私人空间模块是用户个人音乐收藏与播放历史的集中管理区域，承载用户「我喜欢的音乐」「自定义歌单」「专辑/歌手收藏」「播放记录」等核心隐私数据的管理需求。本模块完全遵循产品「数据绝对私有化」的核心定位，所有数据仅存储于本地数据库，支持 WebDAV 端到端加密同步。

### 1.2 功能清单

| 功能点 | 功能描述 | 优先级 |
|---|---|---|
| 我喜欢的歌曲 | 用户一键收藏当前播放歌曲，形成个人喜欢的音乐集合 | P0 |
| 歌单管理 | 用户创建、编辑、删除自定义歌单，支持添加/移除歌曲 | P0 |
| 专辑收藏 | 用户收藏感兴趣的音乐专辑 | P0 |
| 歌手收藏 | 用户关注喜欢的歌手，快速筛选该歌手所有歌曲 | P0 |
| 最近播放 | 记录用户的播放历史，支持按时间倒序查看 | P0 |

---

## 二、数据模型设计

### 2.1 新增数据模型

在 `packages/api/src/models/` 目录下新增 `personal.rs` 文件：

```rust
use serde::{Deserialize, Serialize};

/// 歌单模型
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Playlist {
    pub id: String,              // 唯一标识 (UUID)
    pub name: String,            // 歌单名称
    pub description: String,     // 简介
    pub cover: Option<String>,   // 封面 (默认取第一首歌封面)
    pub track_ids: Vec<String>,  // 歌曲ID列表
    pub track_count: usize,      // 歌曲数量
    pub created_at: i64,         // 创建时间戳
    pub updated_at: i64,         // 更新时间戳
}

/// 播放记录模型
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct PlayHistory {
    pub id: String,              // 唯一标识
    pub track_id: String,        // 歌曲ID
    pub played_at: i64,          // 播放时间戳
}

/// 用户收藏状态汇总
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct UserFavorites {
    pub liked_track_ids: Vec<String>,    // 我喜欢的歌曲ID列表
    pub liked_album_ids: Vec<String>,    // 收藏的专辑ID列表
    pub liked_artist_names: Vec<String>, // 收藏的歌手名列表
}
```

### 2.2 扩展现有模型

#### Track 模型扩展（可选）

在 `Track` 结构体中新增字段：

```rust
pub struct Track {
    // ... 现有字段 ...
    pub is_liked: bool,  // 是否被用户收藏（运行时状态，非持久化）
}
```

### 2.3 数据库表设计

在 `packages/api/src/data/store.rs` 中新增以下表：

```sql
-- 歌单表
CREATE TABLE IF NOT EXISTS playlists (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT DEFAULT '',
    cover TEXT,
    track_count INTEGER DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- 歌单-歌曲关联表
CREATE TABLE IF NOT EXISTS playlist_tracks (
    playlist_id TEXT NOT NULL,
    track_id TEXT NOT NULL,
    position INTEGER NOT NULL,
    added_at INTEGER NOT NULL,
    PRIMARY KEY (playlist_id, track_id),
    FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE,
    FOREIGN KEY (track_id) REFERENCES tracks(id) ON DELETE CASCADE
);

-- 用户收藏表（我喜欢的歌曲）
CREATE TABLE IF NOT EXISTS liked_tracks (
    track_id TEXT PRIMARY KEY,
    liked_at INTEGER NOT NULL,
    FOREIGN KEY (track_id) REFERENCES tracks(id) ON DELETE CASCADE
);

-- 收藏专辑表
CREATE TABLE IF NOT EXISTS liked_albums (
    album_id TEXT PRIMARY KEY,
    album_title TEXT NOT NULL,
    album_artist TEXT NOT NULL,
    album_cover TEXT,
    liked_at INTEGER NOT NULL,
    FOREIGN KEY (album_id) REFERENCES albums(id) ON DELETE CASCADE
);

-- 收藏歌手表
CREATE TABLE IF NOT EXISTS liked_artists (
    artist_name TEXT PRIMARY KEY,
    liked_at INTEGER NOT NULL
);

-- 播放记录表
CREATE TABLE IF NOT EXISTS play_history (
    id TEXT PRIMARY KEY,
    track_id TEXT NOT NULL,
    played_at INTEGER NOT NULL,
    FOREIGN KEY (track_id) REFERENCES tracks(id) ON DELETE CASCADE
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_liked_tracks_liked_at ON liked_tracks(liked_at DESC);
CREATE INDEX IF NOT EXISTS idx_play_history_played_at ON play_history(played_at DESC);
CREATE INDEX IF NOT EXISTS idx_playlist_tracks_position ON playlist_tracks(playlist_id, position);
```

---

## 三、服务层设计

### 3.1 新增 PersonalService

在 `packages/api/src/services/` 目录下新增 `personal.rs` 文件：

```rust
use crate::data::Store;
use crate::models::{PlayHistory, Playlist, Track, UserFavorites};
use chrono::Utc;
use std::collections::HashSet;
use uuid::Uuid;

pub struct PersonalService {
    store: Store,
}

impl PersonalService {
    pub fn new(store: Store) -> Self {
        PersonalService { store }
    }

    // ========== 我喜欢的歌曲 ==========

    /// 收藏歌曲
    pub fn like_track(&self, track_id: &str) -> Result<(), String> {
        self.store.like_track(track_id).map_err(|e| e.to_string())
    }

    /// 取消收藏歌曲
    pub fn unlike_track(&self, track_id: &str) -> Result<(), String> {
        self.store.unlike_track(track_id).map_err(|e| e.to_string())
    }

    /// 检查歌曲是否已收藏
    pub fn is_track_liked(&self, track_id: &str) -> bool {
        self.store.is_track_liked(track_id).unwrap_or(false)
    }

    /// 获取我喜欢的所有歌曲
    pub fn get_liked_tracks(&self) -> Result<Vec<Track>, String> {
        self.store.get_liked_tracks().map_err(|e| e.to_string())
    }

    // ========== 歌单管理 ==========

    /// 创建新歌单
    pub fn create_playlist(&self, name: &str, description: &str) -> Result<Playlist, String> {
        let now = Utc::now().timestamp();
        let playlist = Playlist {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: description.to_string(),
            cover: None,
            track_ids: Vec::new(),
            track_count: 0,
            created_at: now,
            updated_at: now,
        };
        self.store.insert_playlist(&playlist).map_err(|e| e.to_string())?;
        Ok(playlist)
    }

    /// 获取所有歌单
    pub fn get_all_playlists(&self) -> Result<Vec<Playlist>, String> {
        self.store.get_all_playlists().map_err(|e| e.to_string())
    }

    /// 获取指定歌单
    pub fn get_playlist(&self, id: &str) -> Result<Option<Playlist>, String> {
        self.store.get_playlist(id).map_err(|e| e.to_string())
    }

    /// 向歌单添加歌曲
    pub fn add_track_to_playlist(&self, playlist_id: &str, track_id: &str) -> Result<(), String> {
        self.store.add_track_to_playlist(playlist_id, track_id).map_err(|e| e.to_string())?;
        self.refresh_playlist_cover(playlist_id)
    }

    /// 从歌单移除歌曲
    pub fn remove_track_from_playlist(&self, playlist_id: &str, track_id: &str) -> Result<(), String> {
        self.store.remove_track_from_playlist(playlist_id, track_id).map_err(|e| e.to_string())?;
        self.refresh_playlist_cover(playlist_id)
    }

    /// 删除歌单
    pub fn delete_playlist(&self, id: &str) -> Result<(), String> {
        self.store.delete_playlist(id).map_err(|e| e.to_string())
    }

    /// 更新歌单信息
    pub fn update_playlist(&self, id: &str, name: &str, description: &str) -> Result<(), String> {
        self.store.update_playlist(id, name, description).map_err(|e| e.to_string())
    }

    /// 刷新歌单封面（取第一首歌的封面）
    fn refresh_playlist_cover(&self, playlist_id: &str) -> Result<(), String> {
        if let Ok(Some(mut playlist)) = self.store.get_playlist(playlist_id) {
            if let Some(first_track_id) = playlist.track_ids.first() {
                if let Ok(Some(track)) = self.store.get_track(first_track_id) {
                    playlist.cover = track.cover;
                    self.store.update_playlist_cover(playlist_id, &playlist.cover).map_err(|e| e.to_string())?;
                }
            }
        }
        Ok(())
    }

    // ========== 专辑收藏 ==========

    /// 收藏专辑
    pub fn like_album(&self, album_id: &str, title: &str, artist: &str, cover: Option<&str>) -> Result<(), String> {
        self.store.like_album(album_id, title, artist, cover).map_err(|e| e.to_string())
    }

    /// 取消收藏专辑
    pub fn unlike_album(&self, album_id: &str) -> Result<(), String> {
        self.store.unlike_album(album_id).map_err(|e| e.to_string())
    }

    /// 检查专辑是否已收藏
    pub fn is_album_liked(&self, album_id: &str) -> bool {
        self.store.is_album_liked(album_id).unwrap_or(false)
    }

    /// 获取收藏的所有专辑
    pub fn get_liked_albums(&self) -> Result<Vec<LikedAlbum>, String> {
        self.store.get_liked_albums().map_err(|e| e.to_string())
    }

    // ========== 歌手收藏 ==========

    /// 收藏歌手
    pub fn like_artist(&self, artist_name: &str) -> Result<(), String> {
        self.store.like_artist(artist_name).map_err(|e| e.to_string())
    }

    /// 取消收藏歌手
    pub fn unlike_artist(&self, artist_name: &str) -> Result<(), String> {
        self.store.unlike_artist(artist_name).map_err(|e| e.to_string())
    }

    /// 检查歌手是否已收藏
    pub fn is_artist_liked(&self, artist_name: &str) -> bool {
        self.store.is_artist_liked(artist_name).unwrap_or(false)
    }

    /// 获取收藏的所有歌手
    pub fn get_liked_artists(&self) -> Result<Vec<String>, String> {
        self.store.get_liked_artists().map_err(|e| e.to_string())
    }

    // ========== 播放记录 ==========

    /// 记录播放历史
    pub fn add_play_history(&self, track_id: &str) -> Result<(), String> {
        let history = PlayHistory {
            id: Uuid::new_v4().to_string(),
            track_id: track_id.to_string(),
            played_at: Utc::now().timestamp(),
        };
        self.store.add_play_history(&history).map_err(|e| e.to_string())
    }

    /// 获取最近播放记录
    pub fn get_recently_played(&self, limit: usize) -> Result<Vec<Track>, String> {
        self.store.get_recently_played(limit).map_err(|e| e.to_string())
    }

    /// 清空播放历史
    pub fn clear_play_history(&self) -> Result<(), String> {
        self.store.clear_play_history().map_err(|e| e.to_string())
    }

    // ========== 批量操作 ==========

    /// 获取用户收藏状态汇总（用于同步）
    pub fn get_favorites_summary(&self) -> Result<UserFavorites, String> {
        Ok(UserFavorites {
            liked_track_ids: self.store.get_liked_track_ids().map_err(|e| e.to_string())?,
            liked_album_ids: self.store.get_liked_album_ids().map_err(|e| e.to_string())?,
            liked_artist_names: self.store.get_liked_artists().map_err(|e| e.to_string())?,
        })
    }
}
```

### 3.2 Store 层扩展

在 `packages/api/src/data/store.rs` 中新增以下方法：

```rust
// ========== 我喜欢的歌曲 ==========

pub fn like_track(&self, track_id: &str) -> Result<(), StoreError> {
    let now = Utc::now().timestamp();
    self.conn.execute(
        "INSERT OR IGNORE INTO liked_tracks (track_id, liked_at) VALUES (?1, ?2)",
        params![track_id, now],
    )?;
    Ok(())
}

pub fn unlike_track(&self, track_id: &str) -> Result<(), StoreError> {
    self.conn.execute("DELETE FROM liked_tracks WHERE track_id = ?1", params![track_id])?;
    Ok(())
}

pub fn is_track_liked(&self, track_id: &str) -> Result<bool, StoreError> {
    let count: i64 = self.conn.query_row(
        "SELECT COUNT(*) FROM liked_tracks WHERE track_id = ?1",
        params![track_id],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

pub fn get_liked_tracks(&self) -> Result<Vec<Track>, StoreError> {
    let mut stmt = self.conn.prepare(
        "SELECT t.* FROM tracks t
         INNER JOIN liked_tracks lt ON t.id = lt.track_id
         ORDER BY lt.liked_at DESC"
    )?;
    let tracks = stmt.query_map([], |row| Track::from_row(row))?.filter_map(|r| r.ok()).collect();
    Ok(tracks)
}

pub fn get_liked_track_ids(&self) -> Result<Vec<String>, StoreError> {
    let mut stmt = self.conn.prepare("SELECT track_id FROM liked_tracks ORDER BY liked_at DESC")?;
    let ids = stmt.query_map([], |row| row.get(0))?.filter_map(|r| r.ok()).collect();
    Ok(ids)
}

// ========== 歌单管理 ==========

pub fn insert_playlist(&self, playlist: &Playlist) -> Result<(), StoreError> {
    self.conn.execute(
        "INSERT INTO playlists (id, name, description, cover, track_count, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            playlist.id,
            playlist.name,
            playlist.description,
            playlist.cover,
            playlist.track_count as i64,
            playlist.created_at,
            playlist.updated_at,
        ],
    )?;
    Ok(())
}

pub fn get_all_playlists(&self) -> Result<Vec<Playlist>, StoreError> {
    let mut stmt = self.conn.prepare(
        "SELECT id, name, description, cover, track_count, created_at, updated_at
         FROM playlists ORDER BY updated_at DESC"
    )?;
    let playlists = stmt.query_map([], |row| {
        let id: String = row.get(0)?;
        let track_ids = self.get_playlist_track_ids(&id)?;
        Ok(Playlist {
            id,
            name: row.get(1)?,
            description: row.get(2)?,
            cover: row.get(3)?,
            track_ids,
            track_count: row.get::<_, i64>(4)? as usize,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })?.filter_map(|r| r.ok()).collect();
    Ok(playlists)
}

pub fn get_playlist(&self, id: &str) -> Result<Option<Playlist>, StoreError> {
    let mut stmt = self.conn.prepare(
        "SELECT id, name, description, cover, track_count, created_at, updated_at
         FROM playlists WHERE id = ?1"
    )?;
    let playlist = stmt.query_row(params![id], |row| {
        let id: String = row.get(0)?;
        let track_ids = self.get_playlist_track_ids(&id)?;
        Ok(Playlist {
            id,
            name: row.get(1)?,
            description: row.get(2)?,
            cover: row.get(3)?,
            track_ids,
            track_count: row.get::<_, i64>(4)? as usize,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    }).optional()?;
    Ok(playlist)
}

fn get_playlist_track_ids(&self, playlist_id: &str) -> Result<Vec<String>, StoreError> {
    let mut stmt = self.conn.prepare(
        "SELECT track_id FROM playlist_tracks
         WHERE playlist_id = ?1 ORDER BY position ASC"
    )?;
    let ids = stmt.query_map(params![playlist_id], |row| row.get(0))?.filter_map(|r| r.ok()).collect();
    Ok(ids)
}

pub fn add_track_to_playlist(&self, playlist_id: &str, track_id: &str) -> Result<(), StoreError> {
    let now = Utc::now().timestamp();
    let position: i64 = self.conn.query_row(
        "SELECT COALESCE(MAX(position), -1) + 1 FROM playlist_tracks WHERE playlist_id = ?1",
        params![playlist_id],
        |row| row.get(0),
    )?;
    self.conn.execute(
        "INSERT OR IGNORE INTO playlist_tracks (playlist_id, track_id, position, added_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![playlist_id, track_id, position, now],
    )?;
    self.update_playlist_track_count(playlist_id)?;
    Ok(())
}

pub fn remove_track_from_playlist(&self, playlist_id: &str, track_id: &str) -> Result<(), StoreError> {
    self.conn.execute(
        "DELETE FROM playlist_tracks WHERE playlist_id = ?1 AND track_id = ?2",
        params![playlist_id, track_id],
    )?;
    self.update_playlist_track_count(playlist_id)?;
    Ok(())
}

pub fn delete_playlist(&self, id: &str) -> Result<(), StoreError> {
    self.conn.execute("DELETE FROM playlist_tracks WHERE playlist_id = ?1", params![id])?;
    self.conn.execute("DELETE FROM playlists WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn update_playlist(&self, id: &str, name: &str, description: &str) -> Result<(), StoreError> {
    let now = Utc::now().timestamp();
    self.conn.execute(
        "UPDATE playlists SET name = ?1, description = ?2, updated_at = ?3 WHERE id = ?4",
        params![name, description, now, id],
    )?;
    Ok(())
}

pub fn update_playlist_cover(&self, id: &str, cover: &Option<String>) -> Result<(), StoreError> {
    self.conn.execute(
        "UPDATE playlists SET cover = ?1, updated_at = ?2 WHERE id = ?3",
        params![cover, Utc::now().timestamp(), id],
    )?;
    Ok(())
}

fn update_playlist_track_count(&self, playlist_id: &str) -> Result<(), StoreError> {
    let count: i64 = self.conn.query_row(
        "SELECT COUNT(*) FROM playlist_tracks WHERE playlist_id = ?1",
        params![playlist_id],
        |row| row.get(0),
    )?;
    self.conn.execute(
        "UPDATE playlists SET track_count = ?1, updated_at = ?2 WHERE id = ?3",
        params![count, Utc::now().timestamp(), playlist_id],
    )?;
    Ok(())
}

// ========== 专辑收藏 ==========

pub fn like_album(&self, album_id: &str, title: &str, artist: &str, cover: Option<&str>) -> Result<(), StoreError> {
    let now = Utc::now().timestamp();
    self.conn.execute(
        "INSERT OR REPLACE INTO liked_albums (album_id, album_title, album_artist, album_cover, liked_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![album_id, title, artist, cover, now],
    )?;
    Ok(())
}

pub fn unlike_album(&self, album_id: &str) -> Result<(), StoreError> {
    self.conn.execute("DELETE FROM liked_albums WHERE album_id = ?1", params![album_id])?;
    Ok(())
}

pub fn is_album_liked(&self, album_id: &str) -> Result<bool, StoreError> {
    let count: i64 = self.conn.query_row(
        "SELECT COUNT(*) FROM liked_albums WHERE album_id = ?1",
        params![album_id],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

pub fn get_liked_albums(&self) -> Result<Vec<LikedAlbum>, StoreError> {
    let mut stmt = self.conn.prepare(
        "SELECT album_id, album_title, album_artist, album_cover, liked_at
         FROM liked_albums ORDER BY liked_at DESC"
    )?;
    let albums = stmt.query_map([], |row| {
        Ok(LikedAlbum {
            id: row.get(0)?,
            title: row.get(1)?,
            artist: row.get(2)?,
            cover: row.get(3)?,
            liked_at: row.get(4)?,
        })
    })?.filter_map(|r| r.ok()).collect();
    Ok(albums)
}

pub fn get_liked_album_ids(&self) -> Result<Vec<String>, StoreError> {
    let mut stmt = self.conn.prepare("SELECT album_id FROM liked_albums ORDER BY liked_at DESC")?;
    let ids = stmt.query_map([], |row| row.get(0))?.filter_map(|r| r.ok()).collect();
    Ok(ids)
}

// ========== 歌手收藏 ==========

pub fn like_artist(&self, artist_name: &str) -> Result<(), StoreError> {
    let now = Utc::now().timestamp();
    self.conn.execute(
        "INSERT OR IGNORE INTO liked_artists (artist_name, liked_at) VALUES (?1, ?2)",
        params![artist_name, now],
    )?;
    Ok(())
}

pub fn unlike_artist(&self, artist_name: &str) -> Result<(), StoreError> {
    self.conn.execute("DELETE FROM liked_artists WHERE artist_name = ?1", params![artist_name])?;
    Ok(())
}

pub fn is_artist_liked(&self, artist_name: &str) -> Result<bool, StoreError> {
    let count: i64 = self.conn.query_row(
        "SELECT COUNT(*) FROM liked_artists WHERE artist_name = ?1",
        params![artist_name],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

pub fn get_liked_artists(&self) -> Result<Vec<String>, StoreError> {
    let mut stmt = self.conn.prepare("SELECT artist_name FROM liked_artists ORDER BY liked_at DESC")?;
    let artists = stmt.query_map([], |row| row.get(0))?.filter_map(|r| r.ok()).collect();
    Ok(artists)
}

// ========== 播放记录 ==========

pub fn add_play_history(&self, history: &PlayHistory) -> Result<(), StoreError> {
    self.conn.execute(
        "INSERT INTO play_history (id, track_id, played_at) VALUES (?1, ?2, ?3)",
        params![history.id, history.track_id, history.played_at],
    )?;
    Ok(())
}

pub fn get_recently_played(&self, limit: usize) -> Result<Vec<Track>, StoreError> {
    let mut stmt = self.conn.prepare(
        "SELECT DISTINCT t.* FROM tracks t
         INNER JOIN play_history ph ON t.id = ph.track_id
         ORDER BY ph.played_at DESC LIMIT ?1"
    )?;
    let tracks = stmt.query_map(params![limit as i64], |row| Track::from_row(row))?.filter_map(|r| r.ok()).collect();
    Ok(tracks)
}

pub fn clear_play_history(&self) -> Result<(), StoreError> {
    self.conn.execute("DELETE FROM play_history", [])?;
    Ok(())
}
```

---

## 四、状态管理层设计

### 4.1 PersonalProvider

在 `packages/ui/src/state/` 目录下新增 `personal.rs` 文件：

```rust
use crate::models::{Album, Artist, PlayHistory, Playlist, Track};
use dioxus::prelude::*;
use std::collections::HashSet;

pub struct PersonalProvider {
    pub liked_track_ids: Signal<HashSet<String>>,
    pub playlists: Signal<Vec<Playlist>>,
    pub liked_album_ids: Signal<HashSet<String>>,
    pub liked_artists: Signal<HashSet<String>>,
    pub play_history: Signal<Vec<Track>>,
    pub is_loading: Signal<bool>,
}

impl PersonalProvider {
    pub fn init<F>(init_fn: F)
    where
        F: FnOnce() -> PersonalProvider,
    {
        use_context_provider(|| init_fn());
    }

    pub fn like_track(&mut self, track_id: &str) {
        let mut ids = self.liked_track_ids.write();
        ids.insert(track_id.to_string());
    }

    pub fn unlike_track(&mut self, track_id: &str) {
        let mut ids = self.liked_track_ids.write();
        ids.remove(track_id);
    }

    pub fn is_track_liked(&self, track_id: &str) -> bool {
        self.liked_track_ids.read().contains(track_id)
    }

    pub fn add_to_history(&mut self, track: Track) {
        let mut history = self.play_history.write();
        // 移除重复项
        history.retain(|t| t.id != track.id);
        // 添加到开头
        history.insert(0, track);
        // 限制最多保留 100 条
        if history.len() > 100 {
            history.truncate(100);
        }
    }

    pub fn clear_history(&mut self) {
        self.play_history.write().clear();
    }
}
```

### 4.2 模块导出

在 `packages/ui/src/state/mod.rs` 中添加：

```rust
pub mod personal;
pub use personal::PersonalProvider;
```

---

## 五、UI 组件设计

### 5.1 组件目录结构

```
packages/ui/src/components/
├── personal/
│   ├── mod.rs
│   ├── personal_page.rs      # 私人空间主页
│   ├── liked_songs.rs         # 我喜欢的歌曲组件
│   ├── playlist_list.rs      # 歌单列表组件
│   ├── playlist_detail.rs    # 歌单详情组件
│   ├── album_favorites.rs     # 专辑收藏组件
│   ├── artist_favorites.rs    # 歌手收藏组件
│   └── play_history.rs       # 最近播放组件
```

### 5.2 私人空间主页组件

```rust
// packages/ui/src/components/personal/personal_page.rs

use dioxus::prelude::*;

#[component]
pub fn PersonalPage() -> Element {
    let selected_tab = use_signal(|| "liked".to_string());

    rsx! {
        div { class: "flex flex-col h-full bg-white",
            // 顶部导航
            div { class: "flex items-center gap-6 px-8 py-4 border-b border-gray-100",
                button {
                    class: format!(
                        "text-sm font-bold transition-colors {}",
                        if selected_tab() == "liked" { "text-black" } else { "text-gray-400 hover:text-gray-600" }
                    ),
                    onclick: move |_| selected_tab.set("liked".to_string()),
                    "我喜欢的"
                }
                button {
                    class: format!(
                        "text-sm font-bold transition-colors {}",
                        if selected_tab() == "playlists" { "text-black" } else { "text-gray-400 hover:text-gray-600" }
                    ),
                    onclick: move |_| selected_tab.set("playlists".to_string()),
                    "歌单"
                }
                button {
                    class: format!(
                        "text-sm font-bold transition-colors {}",
                        if selected_tab() == "albums" { "text-black" } else { "text-gray-400 hover:text-gray-600" }
                    ),
                    onclick: move |_| selected_tab.set("albums".to_string()),
                    "收藏专辑"
                }
                button {
                    class: format!(
                        "text-sm font-bold transition-colors {}",
                        if selected_tab() == "artists" { "text-black" } else { "text-gray-400 hover:text-gray-600" }
                    ),
                    onclick: move |_| selected_tab.set("artists".to_string()),
                    "关注歌手"
                }
                button {
                    class: format!(
                        "text-sm font-bold transition-colors {}",
                        if selected_tab() == "history" { "text-black" } else { "text-gray-400 hover:text-gray-600" }
                    ),
                    onclick: move |_| selected_tab.set("history".to_string()),
                    "最近播放"
                }
            }

            // 内容区域
            div { class: "flex-1 overflow-y-auto p-8",
                if selected_tab() == "liked" {
                    LikedSongs {}
                } else if selected_tab() == "playlists" {
                    PlaylistList {}
                } else if selected_tab() == "albums" {
                    AlbumFavorites {}
                } else if selected_tab() == "artists" {
                    ArtistFavorites {}
                } else if selected_tab() == "history" {
                    PlayHistoryList {}
                }
            }
        }
    }
}
```

### 5.3 我喜欢的歌曲组件

```rust
// packages/ui/src/components/personal/liked_songs.rs

use crate::state::PlayerProvider;
use crate::components::music_library::TrackTable;
use dioxus::prelude::*;

#[component]
pub fn LikedSongs() -> Element {
    let player = use_context::<PlayerProvider>();
    let liked_tracks = player.liked_tracks.read();
    let total_duration = liked_tracks.iter().map(|t| t.duration as u64).sum::<u64>();

    rsx! {
        div { class: "space-y-6",
            // 头部信息卡片
            div { class: "flex items-end gap-6",
                div { class: "w-48 h-48 rounded-xl bg-gradient-to-br from-pink-500 to-rose-600 flex items-center justify-center shadow-lg",
                    Icon { width: 64, height: 64, icon: LdHeartFill, class: "text-white" }
                }
                div { class: "flex flex-col gap-2",
                    div { class: "text-xs font-bold uppercase tracking-widest text-gray-500", "私人心选" }
                    div { class: "text-4xl font-black", "{liked_tracks.len()} 首歌曲" }
                    div { class: "text-sm text-gray-400", "约 {format_duration(total_duration)}" }
                }
            }

            // 操作栏
            div { class: "flex items-center gap-4 pt-4",
                button {
                    class: "flex items-center gap-2 px-6 py-3 bg-rose-500 text-white font-bold rounded-full hover:bg-rose-600 transition-colors shadow-lg shadow-rose-500/20",
                    onclick: move |_| {
                        spawn(async move {
                            player.play_liked_songs().await;
                        });
                    },
                    Icon { width: 20, height: 20, icon: LdPlay }
                    span { "播放全部" }
                }
                button {
                    class: "flex items-center gap-2 px-4 py-2 text-gray-600 font-bold rounded-full hover:bg-gray-100 transition-colors",
                    onclick: move |_| {
                        player.clear_liked_songs();
                    },
                    Icon { width: 16, height: 16, icon: LdTrash2 }
                    span { "清空列表" }
                }
            }

            // 歌曲列表
            if liked_tracks.is_empty() {
                div { class: "flex flex-col items-center justify-center py-20 text-gray-300",
                    Icon { width: 64, height: 64, icon: LdHeart, class: "opacity-20" }
                    span { class: "text-sm font-bold uppercase tracking-widest mt-4", "还没有收藏任何歌曲" }
                }
            } else {
                TrackTable { tracks: liked_tracks.clone() }
            }
        }
    }
}

fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    if hours > 0 {
        format!("{hours} 小时 {minutes} 分钟")
    } else {
        format!("{minutes} 分钟")
    }
}
```

### 5.4 歌单列表组件

```rust
// packages/ui/src/components/personal/playlist_list.rs

use crate::state::{PersonalProvider, PlayerProvider};
use dioxus::prelude::*;

#[component]
pub fn PlaylistList() -> Element {
    let personal = use_context::<PersonalProvider>();
    let playlists = personal.playlists.read();
    let show_create_modal = use_signal(|| false);

    rsx! {
        div { class: "space-y-6",
            // 头部
            div { class: "flex items-center justify-between",
                div { class: "text-xs font-bold uppercase tracking-widest text-gray-400", "{playlists.len()} 个歌单" }
                button {
                    class: "flex items-center gap-2 px-4 py-2 text-sm font-bold bg-black text-white rounded-full hover:bg-gray-800 transition-colors",
                    onclick: move |_| show_create_modal.set(true),
                    Icon { width: 16, height: 16, icon: LdPlus }
                    span { "新建歌单" }
                }
            }

            // 歌单网格
            if playlists.is_empty() {
                div { class: "flex flex-col items-center justify-center py-20 text-gray-300",
                    Icon { width: 64, height: 64, icon: LdPlaylistMusic, class: "opacity-20" }
                    span { class: "text-sm font-bold uppercase tracking-widest mt-4", "还没有创建任何歌单" }
                }
            } else {
                div { class: "grid grid-cols-4 gap-6",
                    for playlist in playlists.iter() {
                        PlaylistCard { playlist: playlist.clone() }
                    }
                }
            }

            // 创建歌单弹窗
            if show_create_modal() {
                CreatePlaylistModal {
                    visible: show_create_modal
                }
            }
        }
    }
}

#[component]
pub fn PlaylistCard(playlist: Playlist) -> Element {
    let router = use_router();
    let personal = use_context::<PersonalProvider>();

    rsx! {
        div {
            class: "group flex flex-col gap-3 p-4 rounded-xl hover:bg-gray-50 transition-colors cursor-pointer",
            onclick: move |_| {
                router.push_route("/personal/playlist/{playlist.id}");
            },

            // 封面
            div { class: "relative aspect-square rounded-lg bg-gray-100 overflow-hidden",
                if let Some(cover) = &playlist.cover {
                    img { src: "http://covers.localhost/{cover}", class: "w-full h-full object-cover" }
                } else {
                    div { class: "w-full h-full flex items-center justify-center bg-gradient-to-br from-indigo-500 to-purple-600",
                        Icon { width: 32, height: 32, icon: LdMusic, class: "text-white/50" }
                    }
                }
                // 播放量角标
                div { class: "absolute bottom-2 right-2 px-2 py-1 bg-black/50 backdrop-blur-sm rounded text-white text-xs font-bold",
                    "{playlist.track_count} 首"
                }
            }

            // 信息
            div { class: "space-y-1",
                div { class: "text-sm font-bold truncate", "{playlist.name}" }
                div { class: "text-xs text-gray-400 truncate", "{playlist.description}" }
            }
        }
    }
}

#[component]
pub fn CreatePlaylistModal(mut visible: Signal<bool>) -> Element {
    let mut name = use_signal(|| String::new());
    let mut description = use_signal(|| String::new());
    let personal = use_context::<PersonalProvider>();

    rsx! {
        div { class: "fixed inset-0 bg-black/50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-2xl w-[480px] p-8 shadow-2xl",
                div { class: "flex items-center justify-between mb-6",
                    h2 { class: "text-xl font-black", "创建歌单" }
                    button {
                        class: "p-2 text-gray-400 hover:text-black transition-colors",
                        onclick: move |_| visible.set(false),
                        Icon { width: 20, height: 20, icon: LdX }
                    }
                }

                div { class: "space-y-4",
                    div { class: "space-y-2",
                        label { class: "text-sm font-bold text-gray-600", "歌单名称" }
                        input {
                            class: "w-full px-4 py-3 border border-gray-200 rounded-xl text-sm font-medium focus:outline-none focus:border-black focus:ring-1 focus:ring-black transition-colors",
                            placeholder: "给歌单起个名字",
                            value: "{name}",
                            oninput: move |e| name.set(e.value()),
                        }
                    }

                    div { class: "space-y-2",
                        label { class: "text-sm font-bold text-gray-600", "简介（可选）" }
                        textarea {
                            class: "w-full px-4 py-3 border border-gray-200 rounded-xl text-sm font-medium focus:outline-none focus:border-black focus:ring-1 focus:ring-black transition-colors resize-none",
                            placeholder: "写点什么介绍这个歌单",
                            rows: 3,
                            "{description}",
                            oninput: move |e| description.set(e.value()),
                        }
                    }
                }

                div { class: "flex justify-end gap-3 mt-8",
                    button {
                        class: "px-6 py-2.5 text-sm font-bold text-gray-600 hover:bg-gray-100 rounded-full transition-colors",
                        onclick: move |_| visible.set(false),
                        "取消"
                    }
                    button {
                        class: "px-6 py-2.5 text-sm font-bold text-white bg-black rounded-full hover:bg-gray-800 transition-colors disabled:opacity-50 disabled:cursor-not-allowed",
                        disabled: name().trim().is_empty(),
                        onclick: move |_| {
                            spawn(async move {
                                personal.create_playlist(&name(), &description()).await;
                                visible.set(false);
                                name.set(String::new());
                                description.set(String::new());
                            });
                        },
                        "创建"
                    }
                }
            }
        }
    }
}
```

### 5.5 专辑收藏组件

```rust
// packages/ui/src/components/personal/album_favorites.rs

use crate::state::PersonalProvider;
use crate::models::Album;
use dioxus::prelude::*;

#[component]
pub fn AlbumFavorites() -> Element {
    let personal = use_context::<PersonalProvider>();
    let liked_albums = personal.liked_albums.read();

    rsx! {
        div { class: "space-y-6",
            div { class: "text-xs font-bold uppercase tracking-widest text-gray-400",
                "{liked_albums.len()} 张收藏专辑"
            }

            if liked_albums.is_empty() {
                div { class: "flex flex-col items-center justify-center py-20 text-gray-300",
                    Icon { width: 64, height: 64, icon: LdDisc, class: "opacity-20" }
                    span { class: "text-sm font-bold uppercase tracking-widest mt-4", "还没有收藏任何专辑" }
                }
            } else {
                div { class: "grid grid-cols-5 gap-6",
                    for album in liked_albums.iter() {
                        AlbumCard { album: album.clone() }
                    }
                }
            }
        }
    }
}

#[component]
pub fn AlbumCard(album: Album) -> Element {
    let player = use_context::<PlayerProvider>();

    rsx! {
        div {
            class: "group flex flex-col gap-3 cursor-pointer",

            div { class: "relative aspect-square rounded-xl bg-gray-100 overflow-hidden shadow-md group-hover:shadow-xl transition-shadow",
                if let Some(cover) = &album.cover_path {
                    img { src: "http://covers.localhost/{cover}", class: "w-full h-full object-cover" }
                } else {
                    div { class: "w-full h-full flex items-center justify-center bg-gradient-to-br from-amber-500 to-orange-600",
                        Icon { width: 32, height: 32, icon: LdDisc, class: "text-white/50" }
                    }
                }
                // 悬浮播放按钮
                div { class: "absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 flex items-center justify-center transition-opacity",
                    button {
                        class: "w-12 h-12 bg-white rounded-full flex items-center justify-center shadow-lg hover:scale-110 transition-transform",
                        onclick: move |_| {
                            spawn(async move {
                                player.play_album(&album.id).await;
                            });
                        },
                        Icon { width: 24, height: 24, icon: LdPlay, class: "ml-1" }
                    }
                }
            }

            div { class: "space-y-1",
                div { class: "text-sm font-bold truncate", "{album.title}" }
                div { class: "text-xs text-gray-400 truncate", "{album.artist_name}" }
            }
        }
    }
}
```

### 5.6 歌手收藏组件

```rust
// packages/ui/src/components/personal/artist_favorites.rs

use crate::state::{PersonalProvider, PlayerProvider};
use dioxus::prelude::*;

#[component]
pub fn ArtistFavorites() -> Element {
    let personal = use_context::<PersonalProvider>();
    let liked_artists = personal.liked_artists.read();

    rsx! {
        div { class: "space-y-6",
            div { class: "text-xs font-bold uppercase tracking-widest text-gray-400",
                "{liked_artists.len()} 位关注歌手"
            }

            if liked_artists.is_empty() {
                div { class: "flex flex-col items-center justify-center py-20 text-gray-300",
                    Icon { width: 64, height: 64, icon: LdUser, class: "opacity-20" }
                    span { class: "text-sm font-bold uppercase tracking-widest mt-4", "还没有关注任何歌手" }
                }
            } else {
                div { class: "grid grid-cols-4 gap-6",
                    for artist in liked_artists.iter() {
                        ArtistCard { name: artist.clone() }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ArtistCard(name: String) -> Element {
    let personal = use_context::<PersonalProvider>();
    let library = use_context::<LibraryProvider>();

    rsx! {
        div {
            class: "group flex items-center gap-4 p-4 rounded-xl hover:bg-gray-50 transition-colors cursor-pointer",

            div { class: "w-16 h-16 rounded-full bg-gradient-to-br from-violet-500 to-purple-600 flex items-center justify-center flex-shrink-0",
                Icon { width: 24, height: 24, icon: LdUser, class: "text-white" }
            }

            div { class: "flex-1 min-w-0",
                div { class: "text-base font-bold truncate", "{name}" }
                div { class: "text-xs text-gray-400", "歌手" }
            }

            button {
                class: "p-2 text-gray-400 hover:text-red-500 transition-colors opacity-0 group-hover:opacity-100",
                onclick: move |e| {
                    e.stop_propagation();
                    spawn(async move {
                        personal.unlike_artist(&name).await;
                    });
                },
                Icon { width: 16, height: 16, icon: LdHeartFill, class: "text-red-500" }
            }
        }
    }
}
```

### 5.7 最近播放组件

```rust
// packages/ui/src/components/personal/play_history.rs

use crate::state::{PersonalProvider, PlayerProvider};
use dioxus::prelude::*;

#[component]
pub fn PlayHistoryList() -> Element {
    let personal = use_context::<PersonalProvider>();
    let history = personal.play_history.read();

    rsx! {
        div { class: "space-y-6",
            // 头部
            div { class: "flex items-center justify-between",
                div { class: "text-xs font-bold uppercase tracking-widest text-gray-400",
                    "最近播放 ({history.len()} 首)"
                }
                button {
                    class: "flex items-center gap-2 px-3 py-1.5 text-xs font-bold text-gray-400 hover:text-black hover:bg-gray-100 rounded-full transition-colors",
                    onclick: move |_| {
                        personal.clear_history();
                    },
                    Icon { width: 14, height: 14, icon: LdTrash2 }
                    span { "清空记录" }
                }
            }

            if history.is_empty() {
                div { class: "flex flex-col items-center justify-center py-20 text-gray-300",
                    Icon { width: 64, height: 64, icon: LdHistory, class: "opacity-20" }
                    span { class: "text-sm font-bold uppercase tracking-widest mt-4", "还没有播放记录" }
                }
            } else {
                div { class: "space-y-1",
                    for track in history.iter() {
                        HistoryTrackItem { track: track.clone() }
                    }
                }
            }
        }
    }
}

#[component]
pub fn HistoryTrackItem(track: Track) -> Element {
    let player = use_context::<PlayerProvider>();

    rsx! {
        div {
            class: "group flex items-center gap-4 px-4 py-3 rounded-xl hover:bg-gray-50 transition-colors cursor-pointer",

            // 封面
            div { class: "w-12 h-12 rounded-lg bg-gray-100 flex-shrink-0 overflow-hidden",
                if let Some(cover) = &track.cover {
                    img { src: "http://covers.localhost/{cover}", class: "w-full h-full object-cover" }
                } else {
                    Icon { width: 20, height: 20, icon: LdMusic, class: "text-gray-300 m-auto" }
                }
            }

            // 信息
            div { class: "flex-1 min-w-0",
                div { class: "text-sm font-bold truncate", "{track.title}" }
                div { class: "text-xs text-gray-400 truncate", "{track.artist}" }
            }

            // 时长
            div { class: "text-xs font-mono text-gray-300", "{track.duration}" }
        }
    }
}
```

---

## 六、集成到播放器

### 6.1 播放时自动记录历史

在 `PlayerService` 的 `play_track` 方法中调用记录接口：

```rust
// packages/api/src/services/player.rs

pub async fn play_track(&mut self, track_id: &str) -> Result<(), String> {
    // ... 现有播放逻辑 ...

    // 记录播放历史
    self.personal_service.add_play_history(track_id).map_err(|e| e.to_string())?;

    Ok(())
}
```

### 6.2 一键收藏/取消收藏

在 `PlayerBar` 组件中添加收藏按钮：

```rust
// packages/ui/src/components/player/player_bar.rs

let is_liked = use_memo(move || {
    player.is_track_liked(&current_track.id)
});

button {
    class: format!(
        "p-2 transition-colors {}",
        if is_liked() { "text-rose-500" } else { "text-gray-400 hover:text-gray-600" }
    ),
    onclick: move |_| {
        spawn(async move {
            if is_liked() {
                player.unlike_track(&current_track.id).await;
            } else {
                player.like_track(&current_track.id).await;
            }
        });
    },
    Icon {
        width: 20, height: 20,
        icon: if is_liked() { LdHeartFill } else { LdHeart }
    }
}
```

---

## 七、文件清单汇总

| 文件路径 | 操作 | 说明 |
|---|---|---|
| `packages/api/src/models/personal.rs` | 新增 | 个人空间数据模型 |
| `packages/api/src/services/personal.rs` | 新增 | 个人空间服务层 |
| `packages/api/src/data/store.rs` | 修改 | 新增数据库表和方法 |
| `packages/api/src/lib.rs` | 修改 | 导出 PersonalService |
| `packages/ui/src/state/personal.rs` | 新增 | 个人空间状态管理 |
| `packages/ui/src/state/mod.rs` | 修改 | 导出 PersonalProvider |
| `packages/ui/src/components/personal/mod.rs` | 新增 | 组件模块入口 |
| `packages/ui/src/components/personal/*.rs` | 新增 | 各组件实现 |
| `packages/ui/src/components/mod.rs` | 修改 | 导出 personal 模块 |
| `packages/desktop/src/main.rs` | 修改 | 初始化 PersonalService |

---

## 八、实现优先级与排期建议

| 阶段 | 功能点 | 建议工时 |
|---|---|---|
| **Phase 1** | 我喜欢的歌曲 + 最近播放 | 4h |
| **Phase 2** | 歌单管理（创建/删除/编辑） | 6h |
| **Phase 3** | 专辑收藏 + 歌手收藏 | 4h |
| **Phase 4** | 私人空间主页 UI + 路由集成 | 3h |
| **Phase 5** | 与播放器深度集成（播放历史自动记录、一键收藏） | 3h |
| **合计** | | **20h** |
