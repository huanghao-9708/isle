use crate::data::{Cache, Store};
use crate::infrastructure::FileSystem;
use crate::models::{Album, Artist, FolderSummary, GenreSummary, LibraryStats, ScanFolder, Track};
use chrono::Utc;
use lofty::file::TaggedFileExt;
use lofty::prelude::*;
use lofty::probe::Probe;
use lofty::tag::ItemKey;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
#[allow(unused_imports)]
use tracing::{error, info, warn};

/// 解析文件名，尝试提取标题和艺术家。
///
/// # 参数
/// * `filename` - 原始文件名，例如 "周杰伦 - 七里香.mp3"
///
/// # 返回
/// 包含 (标题, 艺术家) 的元组
fn parse_filename(filename: &str) -> (String, String) {
    let parts: Vec<&str> = filename.split(" - ").collect();
    if parts.len() >= 2 {
        let artist = parts[0].trim().to_string();
        let title_with_ext = parts[1..].join(" - ").trim().to_string();
        let title = std::path::Path::new(&title_with_ext)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&title_with_ext)
            .to_string();
        (title, artist)
    } else {
        let title = std::path::Path::new(filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(filename)
            .to_string();
        (title, "未知艺术家".to_string())
    }
}

/// 计算字符串的 SHA-256 哈希值，用于生成唯一 ID。
///
/// # 参数
/// * `input` - 输入字符串
fn compute_string_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// 获取文件的后缀名（小写）。
fn get_file_extension(path: &str) -> String {
    std::path::Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase()
}

/// 获取文件的最后修改时间（Unix 时间戳）。
fn get_file_mtime(path: &str) -> Result<i64, String> {
    let meta = fs::metadata(path).map_err(|e| e.to_string())?;
    let modified = meta.modified().map_err(|e| e.to_string())?;
    let duration_since_epoch = modified
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?;
    Ok(duration_since_epoch.as_secs() as i64)
}

/// 音乐库管理服务。
///
/// 负责音频扫描、元数据提取、数据库同步以及分类摘要查询。
pub struct LibraryService {
    fs: FileSystem,
    pub store: Option<Store>,
    cache: Cache,
    pub scan_progress: f32,
    pub stats: LibraryStats,
    cover_dir: Option<PathBuf>,
}

impl Default for LibraryService {
    fn default() -> Self {
        Self::new()
    }
}

impl LibraryService {
    /// 创建一个新的音乐库服务实例。
    pub fn new() -> Self {
        LibraryService {
            fs: FileSystem::new(),
            store: None,
            cache: Cache::new(),
            scan_progress: 0.0,
            stats: LibraryStats {
                track_count: 0,
                album_count: 0,
                artist_count: 0,
                genre_count: 0,
                folder_count: 0,
            },
            cover_dir: None,
        }
    }

    /// 设置封面缓存目录。
    pub fn set_cover_dir(&mut self, path: PathBuf) {
        self.cover_dir = Some(path);
    }

    /// 获取当前封面缓存目录。
    pub fn cover_dir(&self) -> Option<PathBuf> {
        self.cover_dir.clone()
    }

    /// 初始化底层数据库上下文。
    pub fn init_database(&mut self, db_path: PathBuf) -> Result<(), String> {
        let store = Store::new(db_path).map_err(|e| e.to_string())?;
        self.store = Some(store);
        self.refresh_stats()
    }

    /// 强制从数据库刷新全库统计信息。
    pub fn refresh_stats(&mut self) -> Result<(), String> {
        let Some(store) = &self.store else {
            return Err("Database not initialized".to_string());
        };
        self.stats.track_count = store.get_track_count().map_err(|e| e.to_string())?;
        self.stats.album_count = store.get_album_count().map_err(|e| e.to_string())?;
        self.stats.artist_count = store.get_artist_count().map_err(|e| e.to_string())?;
        self.stats.genre_count = store.get_genre_count().map_err(|e| e.to_string())?;
        self.stats.folder_count = store.get_scan_folder_count().map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 获取指定目录下的音频文件列表 (非阻塞，纯文件系统遍历)。
    ///
    /// # 参数
    /// * `path` - 目录路径，例如 "D:/Music"
    pub fn get_audio_files(
        &self,
        path: &str,
    ) -> Result<Vec<crate::infrastructure::AudioFile>, String> {
        self.fs.scan_folder(path)
    }

    /// 批量同步音轨到数据库、更新缓存并刷新全库统计信息。
    ///
    /// # 参数
    /// * `tracks` - 待写入的音轨列表
    /// * `path` - 该批次归属的顶级扫描目录路径
    pub fn sync_tracks_batch(&mut self, tracks: &[Track], path: &str) -> Result<(), String> {
        let Some(store) = &self.store else {
            return Err("Database not initialized".to_string());
        };

        if !tracks.is_empty() {
            store
                .insert_track_batch(tracks)
                .map_err(|e| e.to_string())?;
            for track in tracks {
                self.cache.set_track(track.clone());
            }
        }

        store
            .update_folder_scan_time(path, Utc::now().timestamp())
            .map_err(|e| e.to_string())?;

        // 刷新统计数据缓存
        self.refresh_stats()?;

        Ok(())
    }

    /// 扫描指定文件夹下的所有音频文件并将其同步到数据库。
    ///
    /// 性能优化：
    /// 1. 并发处理：利用 tokio JoinSet 并行解析文件元数据。
    /// 2. 增量更新：比对文件 mtime，跳过未修改的文件。
    pub async fn scan_folder(&mut self, path: &str) -> Result<(), String> {
        let normalized_path = path.replace('\\', "/");
        info!("LibraryService: 开始增量并发扫描 -> {}", normalized_path);

        // 1. 获取库中已有的文件 mtime 映射
        let mtime_map = if let Some(store) = &self.store {
            store
                .get_path_mtime_map(&normalized_path)
                .unwrap_or_default()
        } else {
            std::collections::HashMap::new()
        };

        // 2. 发现物理文件
        let audio_files = self.get_audio_files(&normalized_path)?;
        let total = audio_files.len();
        let mut to_process = Vec::new();

        for file in audio_files {
            let file_path_str = file.path.to_string_lossy().to_string();
            let mtime = get_file_mtime(&file_path_str).unwrap_or(0);

            // 增量检查：如果 mtime 没变，则跳过解析
            if let Some(&old_mtime) = mtime_map.get(&file_path_str) {
                if old_mtime == mtime {
                    continue;
                }
            }
            to_process.push(file);
        }

        let skipped = total - to_process.len();
        info!(
            "LibraryService: {} 个文件未变动，跳过解析。需要解析 {}/{} 个文件。",
            skipped,
            to_process.len(),
            total
        );

        if to_process.is_empty() {
            return Ok(());
        }

        // 3. 并发解析元数据
        let mut join_set = tokio::task::JoinSet::new();

        // 限制并发数以防句柄耗尽或内存溢出
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(16));

        for file in to_process {
            let sem = semaphore.clone();
            let cover_dir = self.cover_dir.clone();

            // 我们不能直接调用 extract_track_metadata 因为它需要 &self
            // 提取关键逻辑到异步闭包中
            join_set.spawn(async move {
                let _permit = sem.acquire().await.ok();
                Self::extract_metadata_standalone(file, cover_dir).await
            });
        }

        let mut processed_count = skipped;
        let mut batch = Vec::new();

        while let Some(res) = join_set.join_next().await {
            if let Ok(Some(track)) = res {
                batch.push(track);
            }

            processed_count += 1;
            self.scan_progress = processed_count as f32 / total as f32;

            if batch.len() >= 500 {
                info!(
                    "LibraryService: 正在同步 {} 条记录到数据库 (进度: {:.1}%)...",
                    batch.len(),
                    self.scan_progress * 100.0
                );
                self.sync_tracks_batch(&batch, &normalized_path)?;
                batch.clear();

                // 让出执行权，避免大规模解析导致后台线程堵塞，防止前端无响应
                tokio::task::yield_now().await;
            }
        }

        // 4. 同步剩余的尾部数据
        if !batch.is_empty() {
            info!(
                "LibraryService: 正在同步最后 {} 条记录到数据库...",
                batch.len()
            );
            self.sync_tracks_batch(&batch, &normalized_path)?;
        }

        self.scan_progress = 1.0;
        Ok(())
    }

    /// 提取元数据 (静态方法，方便并发调用)
    pub async fn extract_metadata_standalone(
        audio_file: crate::infrastructure::AudioFile,
        cover_dir: Option<PathBuf>,
    ) -> Option<Track> {
        let file_path_str = audio_file.path.to_string_lossy().to_string();
        let file_name = audio_file.file_name.clone();

        tokio::task::spawn_blocking(move || {
            // 使用快速哈希
            let fs = FileSystem::new();
            let hash = match fs.compute_quick_hash(Path::new(&file_path_str)) {
                Ok(h) => h,
                Err(e) => {
                    warn!("Failed to compute hash for {}: {}", file_path_str, e);
                    return None;
                }
            };

            let mtime = get_file_mtime(&file_path_str).unwrap_or(0);
            let size = audio_file.file_size;
            let (mut title, mut artist) = parse_filename(&file_name);
            let mut album = "未知专辑".to_string();
            let mut duration = 0;
            let mut genres = Vec::new();
            let mut cover_data: Option<(Vec<u8>, String)> = None;
            let mut lyrics = None;

            if let Ok(tagged_file) = Probe::open(&file_path_str).and_then(|p| p.read()) {
                let properties = tagged_file.properties();
                duration = properties.duration().as_secs() as u32;

                if let Some(tag) = tagged_file
                    .primary_tag()
                    .or_else(|| tagged_file.first_tag())
                {
                    if let Some(t) = tag.title().map(|s| s.to_string()) {
                        title = t;
                    }
                    if let Some(a) = tag.artist().map(|s| s.to_string()) {
                        artist = a;
                    }
                    if let Some(al) = tag.album().map(|s| s.to_string()) {
                        album = al;
                    }

                    if let Some(picture) = tag.pictures().first() {
                        cover_data = Some((
                            picture.data().to_vec(),
                            picture
                                .mime_type()
                                .map(|m| m.to_string())
                                .unwrap_or_else(|| "image/jpeg".to_string()),
                        ));
                    }

                    for g_str in tag.get_strings(&ItemKey::Genre) {
                        for g in g_str
                            .split(['/', ';', ','])
                            .map(|s| s.trim())
                            .filter(|s| !s.is_empty())
                        {
                            if !genres.contains(&g.to_string()) {
                                genres.push(g.to_string());
                            }
                        }
                    }

                    // 提取内置歌词
                    if let Some(l) = tag.get_strings(&ItemKey::Lyrics).next() {
                        lyrics = Some(l.to_string());
                    }
                }
            }

            // 如果标签中没有歌词，尝试查找同名 .lrc 文件
            if lyrics.is_none() {
                let audio_path = Path::new(&file_path_str);
                if let Some(parent) = audio_path.parent() {
                    let lrc_path = parent
                        .join(audio_path.file_stem().unwrap())
                        .with_extension("lrc");
                    if lrc_path.exists() {
                        if let Ok(content) = std::fs::read_to_string(lrc_path) {
                            lyrics = Some(content);
                        }
                    }
                }
            }

            let album_id = compute_string_hash(&format!("{}{}", artist, album));
            let artist_id = compute_string_hash(&artist);

            let mut cover_path_str = None;
            if let Some((data, mime)) = cover_data {
                if let Some(cdir) = &cover_dir {
                    let ext = if mime.to_lowercase().contains("png") {
                        "png"
                    } else {
                        "jpg"
                    };
                    let filename = format!("album_{}.{}", album_id, ext);
                    let target_path = cdir.join(&filename);
                    if !target_path.exists() {
                        let _ = std::fs::write(&target_path, data);
                    }
                    cover_path_str = Some(filename);
                }
            }

            Some(Track {
                id: hash,
                path: file_path_str,
                title,
                artist,
                artist_id,
                album,
                album_id,
                duration,
                size,
                bitrate: None,
                extension: get_file_extension(&file_name),
                genres,
                added_at: Utc::now().timestamp(),
                mtime,
                cover: cover_path_str,
                lyrics,
                played_at: None,
            })
        })
        .await
        .unwrap_or(None)
    }

    /// 获取库中所有的音轨。
    pub fn get_all_tracks(&self) -> Result<Vec<Track>, String> {
        let Some(store) = &self.store else {
            return Ok(vec![]);
        };
        store.get_all_tracks().map_err(|e| e.to_string())
    }

    /// 根据 ID 获取单个音轨信息。优先从缓存读取。
    pub fn get_track(&self, id: &str) -> Result<Option<Track>, String> {
        if let Some(track) = self.cache.get_track(id) {
            Ok(Some(track.clone()))
        } else if let Some(store) = &self.store {
            store.get_track(id).map_err(|e| e.to_string())
        } else {
            Ok(None)
        }
    }

    /// 获取完整包含歌词的音轨信息，直接击穿缓存去数据库拉取
    pub fn get_track_with_lyrics(&self, id: &str) -> Result<Option<Track>, String> {
        if let Some(store) = &self.store {
            store.get_track(id).map_err(|e| e.to_string())
        } else {
            Ok(None)
        }
    }

    /// 从数据库和缓存中物理删除指定音轨。
    pub fn delete_track(&mut self, id: &str) -> Result<(), String> {
        self.cache.remove_track(id);
        if let Some(store) = &self.store {
            store.delete_track(id).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// 向库中添加一个新的扫描目录。
    pub async fn add_scan_folder(&mut self, path: &str, enabled: bool) -> Result<(), String> {
        let Some(store) = &self.store else {
            return Err("Database not initialized".to_string());
        };
        let normalized_path = path.replace('\\', "/");
        store
            .add_scan_folder(&normalized_path, enabled)
            .map_err(|e| e.to_string())?;

        if enabled {
            self.scan_folder(&normalized_path).await?;
        }

        Ok(())
    }

    /// 更新扫描目录的启用状态。
    pub fn update_scan_folder_status(&self, id: &str, enabled: bool) -> Result<(), String> {
        let Some(store) = &self.store else {
            return Err("Database not initialized".to_string());
        };
        store
            .update_scan_folder_enabled(id, enabled)
            .map_err(|e| e.to_string())
    }

    /// 仅清理特定目录下的所有音轨数据和缓存，但【保留】该扫描目录的记录。
    /// 适用于“禁用（取消勾选）目录”而非“完全移除目录”的场景。
    pub fn purge_scan_folder_tracks(&mut self, path: &str) -> Result<(), String> {
        let Some(store) = &self.store else {
            return Err("Database not initialized".to_string());
        };
        let normalized_path = path.replace('\\', "/");

        // 1. 从数据库物理删除关联音轨
        store
            .delete_tracks_by_path_prefix(&normalized_path)
            .map_err(|e| e.to_string())?;

        // 2. 同步清理内存缓存
        self.cache.remove_tracks_by_path_prefix(&normalized_path);

        // 3. 更新全库统计信息
        self.refresh_stats()
    }

    /// 批量同步扫描目录变更。
    ///
    /// 会根据传入的列表与当前库中状态进行对比，执行差异更新（新增、删除、状态切换）。
    /// 返回需要执行全量或增量扫描的目录路径列表。
    pub async fn batch_sync_folders(
        &mut self,
        new_folders: Vec<(String, String, bool)>,
    ) -> Result<Vec<String>, String> {
        if self.store.is_none() {
            return Err("Database not initialized".to_string());
        }

        let mut paths_to_scan = Vec::new();

        // 0. 预处理：统一对 UI 传来的路径进行归一化，防止目录格式差异导致的比对失效
        let normalized_new_folders: Vec<(String, String, bool)> = new_folders
            .into_iter()
            .map(|(id, path, enabled)| (id, path.replace('\\', "/"), enabled))
            .collect();

        let current_folders = self.get_scan_folders()?;

        // 1. 处理删除：如果旧的有，新的没有，则执行物理移除
        for current in &current_folders {
            if !normalized_new_folders
                .iter()
                .any(|(_, path, _)| path == &current.path)
            {
                info!("LibraryService: 检测到目录移除 -> {}", current.path);
                let _ = self.remove_scan_folder(&current.path);
            }
        }

        // 2. 处理新增和状态更新

        for (_, path, enabled) in normalized_new_folders {
            let existing = current_folders.iter().find(|f| f.path == path);

            match existing {
                Some(ext) => {
                    if ext.enabled != enabled {
                        info!(
                            "LibraryService: 更新目录状态 -> {} (enabled: {})",
                            path, enabled
                        );
                        // 更新启用状态 (重新从 self 获取 store 引用)
                        if let Some(store) = &self.store {
                            if let Err(e) = store.update_scan_folder_enabled(&ext.id, enabled) {
                                error!("LibraryService: 更新目录状态失败: {}", e);
                                continue;
                            }
                        }

                        if !enabled {
                            info!("LibraryService: 目录已禁用，清理关联音轨数据: {}", path);
                            let _ = self.purge_scan_folder_tracks(&path);
                        } else {
                            info!("LibraryService: 目录已重新启用，记录待扫描路径: {}", path);
                            paths_to_scan.push(path);
                        }
                    }
                }
                None => {
                    // 全新目录
                    info!(
                        "LibraryService: 发现新记录，准备持久化目录 -> {} (enabled: {})",
                        path, enabled
                    );
                    let added = if let Some(store) = &self.store {
                        if let Err(e) = store.add_scan_folder(&path, enabled) {
                            error!("LibraryService: 新增目录持久化失败: {}", e);
                            false
                        } else {
                            true
                        }
                    } else {
                        false
                    };

                    if added && enabled {
                        info!("LibraryService: 新目录已启用，记录待扫描路径: {}", path);
                        paths_to_scan.push(path);
                    }
                }
            }
        }

        self.refresh_stats()?;
        Ok(paths_to_scan)
    }

    /// 移除扫描目录，并物理删除库中该目录下所有关联的音轨数据。
    pub fn remove_scan_folder(&mut self, path: &str) -> Result<(), String> {
        let Some(store) = &self.store else {
            return Err("Database not initialized".to_string());
        };
        let normalized_path = path.replace('\\', "/");
        info!("LibraryService: 准备移除扫描目录 -> {}", normalized_path);

        // 1. 从数据库移除目录记录
        store
            .remove_scan_folder(&normalized_path)
            .map_err(|e| e.to_string())?;

        // 2. 从数据库删除关联音轨
        store
            .delete_tracks_by_path_prefix(&normalized_path)
            .map_err(|e| e.to_string())?;

        // 3. 同步清理内存缓存
        self.cache.remove_tracks_by_path_prefix(&normalized_path);

        // 4. 重置全库统计信息
        self.refresh_stats()?;

        Ok(())
    }

    /// 获取当前库中已配置的所有扫描目录。
    pub fn get_scan_folders(&self) -> Result<Vec<ScanFolder>, String> {
        let Some(store) = &self.store else {
            return Ok(vec![]);
        };
        store.get_scan_folders().map_err(|e| e.to_string())
    }

    /// 获取当前的扫描进度百分比 (0.0 - 1.0)。
    pub fn scan_progress(&self) -> f32 {
        self.scan_progress
    }

    /// 获取当前的库统计摘要信息。
    pub fn stats(&self) -> &LibraryStats {
        &self.stats
    }

    /// 关键字模糊搜索音轨，支持歌名或艺术家。
    ///
    /// # 示例
    /// ```rust
    /// let results = library.search_tracks("周杰伦", 1, 20)?;
    /// ```
    pub fn search_tracks(
        &self,
        keyword: &str,
        page: usize,
        page_size: usize,
    ) -> Result<crate::models::PaginatedResult<Track>, String> {
        let Some(store) = &self.store else {
            return Err("Database not initialized".to_string());
        };
        let offset = (page.max(1) - 1) * page_size;
        let (items, total) = store
            .search_tracks(keyword, page_size, offset)
            .map_err(|e| e.to_string())?;
        Ok(crate::models::PaginatedResult {
            items,
            total,
            page,
            page_size,
        })
    }

    /// 根据详细条件（艺术家、专辑、流派、路径）过滤音轨。
    ///
    /// # 示例
    /// ```rust
    /// let filter = TrackFilter { artist: Some("周杰伦".into()), ..Default::default() };
    /// let items = library.filter_tracks(filter, 1, 20)?;
    /// ```
    pub fn filter_tracks(
        &self,
        filter: crate::models::TrackFilter,
        page: usize,
        page_size: usize,
    ) -> Result<crate::models::PaginatedResult<Track>, String> {
        let Some(store) = &self.store else {
            return Err("Database not initialized".to_string());
        };
        let offset = (page.max(1) - 1) * page_size;
        let (items, total) = store
            .filter_tracks(filter, page_size, offset)
            .map_err(|e| e.to_string())?;
        Ok(crate::models::PaginatedResult {
            items,
            total,
            page,
            page_size,
        })
    }

    /// 获取库中唯一艺术家列表及其统计。
    pub fn get_artist_summaries(&self) -> Result<Vec<Artist>, String> {
        self.store
            .as_ref()
            .ok_or("Database not initialized")?
            .get_artist_summaries()
            .map_err(|e| e.to_string())
    }

    /// 获取库中唯一专辑列表及其统计。
    pub fn get_album_summaries(&self) -> Result<Vec<Album>, String> {
        self.store
            .as_ref()
            .ok_or("Database not initialized")?
            .get_album_summaries()
            .map_err(|e| e.to_string())
    }

    pub fn get_albums_by_artist(&self, artist_id: &str) -> Result<Vec<Album>, String> {
        self.store
            .as_ref()
            .ok_or("Database not initialized")?
            .get_albums_by_artist(artist_id)
            .map_err(|e| e.to_string())
    }

    /// 获取库中唯一流派列表及其统计。
    pub fn get_genre_summaries(&self) -> Result<Vec<GenreSummary>, String> {
        self.store
            .as_ref()
            .ok_or("Database not initialized")?
            .get_genre_summaries()
            .map_err(|e| e.to_string())
    }

    /// 根据物理路径层级获取文件夹摘要统计。
    pub fn get_folder_summaries(&self) -> Result<Vec<FolderSummary>, String> {
        self.store
            .as_ref()
            .ok_or("Database not initialized")?
            .get_folder_summaries()
            .map_err(|e| e.to_string())
    }

    pub fn get_artist(&self, id: &str) -> Result<Option<Artist>, String> {
        self.store
            .as_ref()
            .ok_or("Database not initialized")?
            .get_artist(id)
            .map_err(|e| e.to_string())
    }

    pub fn get_album(&self, id: &str) -> Result<Option<Album>, String> {
        self.store
            .as_ref()
            .ok_or("Database not initialized")?
            .get_album(id)
            .map_err(|e| e.to_string())
    }

    pub fn get_genre(&self, id: i32) -> Result<Option<GenreSummary>, String> {
        self.store
            .as_ref()
            .ok_or("Database not initialized")?
            .get_genre(id)
            .map_err(|e| e.to_string())
    }
}
