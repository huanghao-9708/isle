use crate::components::music_library::ScanFolder as UIScanFolder;
use crate::components::types::Track as UITrack;
use api::models::{
    Album as ApiAlbum, Artist as ApiArtist, FolderSummary as ApiFolderSummary,
    GenreSummary as ApiGenreSummary, PaginatedResult, Track as ApiTrack, TrackFilter,
    UserPlaylist as ApiPlaylist,
};

#[derive(Clone, PartialEq, Debug)]
pub enum LibraryView {
    Main,
    ArtistDetail(ApiArtist),
    AlbumDetail(ApiAlbum),
    GenreDetail(ApiGenreSummary),
    // 私人空间子视图
    PersonalOverview,
    PersonalLiked,
    PersonalPlaylists,
    PersonalPlaylistsCreate,
    PlaylistDetail(ApiPlaylist),
    PersonalAlbums,
    PersonalArtists,
    PersonalRecent,
    /// 全量歌曲详情页 (通用，基于过滤条件)
    AllTracksDetail {
        title: String,
        filter: TrackFilter,
        /// 特殊标识，用于区分是否是“我喜欢的”或者“歌单”
        source_type: String,
    },
}
use api::services::LibraryService;
use dioxus::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

/// 音乐库全局状态包装类
///
/// 该结构体利用 Dioxus 的 Signal 机制包装了底层的 LibraryService，
/// 使得 UI 组件可以直接消费响应式的数据流。
#[derive(Clone, Copy)]
pub struct LibraryProvider {
    service: Signal<Arc<Mutex<LibraryService>>>,
    pub folders: Signal<Vec<UIScanFolder>>,
    pub is_scanning: Signal<bool>,
    pub tracks: Signal<Vec<UITrack>>,
    /// 当前主视图原始音轨数据 (用于播放队列同步)
    pub current_api_tracks: Signal<Vec<ApiTrack>>,
    pub track_count: Signal<usize>,
    pub album_count: Signal<usize>,
    pub artist_count: Signal<usize>,
    pub genre_count: Signal<usize>,
    pub folder_count: Signal<usize>,
    pub artists: Signal<Vec<ApiArtist>>,
    pub albums: Signal<Vec<ApiAlbum>>,
    pub genres: Signal<Vec<ApiGenreSummary>>,
    pub folder_summaries: Signal<Vec<ApiFolderSummary>>,
    pub filter: Signal<TrackFilter>,
    /// 原始后端音轨模型缓存
    pub all_api_tracks: Signal<Vec<ApiTrack>>,
    /// 封面图片存储目录
    pub cover_dir: Signal<Option<std::path::PathBuf>>,
    /// 当前视图
    pub current_view: Signal<LibraryView>,
    /// 详情页音轨列表
    pub detail_tracks: Signal<Vec<UITrack>>,
    /// 详情页原始音轨数据
    pub detail_api_tracks: Signal<Vec<ApiTrack>>,
    /// 详情页专辑列表
    pub detail_albums: Signal<Vec<ApiAlbum>>,
    /// 导航历史栈
    pub navigation_history: Signal<Vec<LibraryView>>,
    /// 当前全局激活的 Tab (library, personal, discover, settings)
    pub active_global_tab: Signal<String>,
}

impl LibraryProvider {
    /// 创建一个新的提供者（非钩子版本，用于初始化）
    pub fn new(service: LibraryService) -> Self {
        let cover_dir = service.cover_dir();
        Self {
            service: Signal::new(Arc::new(Mutex::new(service))),
            folders: Signal::new(Vec::new()),
            is_scanning: Signal::new(false),
            tracks: Signal::new(Vec::new()),
            current_api_tracks: Signal::new(Vec::new()),
            track_count: Signal::new(0),
            album_count: Signal::new(0),
            artist_count: Signal::new(0),
            genre_count: Signal::new(0),
            folder_count: Signal::new(0),
            artists: Signal::new(Vec::new()),
            albums: Signal::new(Vec::new()),
            genres: Signal::new(Vec::new()),
            folder_summaries: Signal::new(Vec::new()),
            filter: Signal::new(TrackFilter::default()),
            all_api_tracks: Signal::new(Vec::new()),
            cover_dir: Signal::new(cover_dir),
            current_view: Signal::new(LibraryView::Main),
            detail_tracks: Signal::new(Vec::new()),
            detail_api_tracks: Signal::new(Vec::new()),
            detail_albums: Signal::new(Vec::new()),
            navigation_history: Signal::new(Vec::new()),
            active_global_tab: Signal::new("library".to_string()),
        }
    }

    /// 初始化提供者并注入到 Context 中
    pub fn init(service_factory: impl FnOnce() -> LibraryService) -> Self {
        let provider = use_hook(|| Self::new(service_factory()));
        use_context_provider(|| provider);

        // 初始加载文件夹列表、音轨和统计信息
        let mut provider_clone = provider;
        use_resource(move || async move {
            provider_clone.refresh_all().await;
        });

        provider
    }

    /// 刷新所有数据
    pub async fn refresh_all(&mut self) {
        info!("LibraryProvider: 正在执行全量数据刷新...");
        self.refresh_folders().await;
        self.refresh_stats().await;
        self.refresh_summaries().await;
        self.refresh_tracks().await;
        info!("LibraryProvider: 全量数据刷新完成");
    }

    /// 刷新统计信息
    pub async fn refresh_stats(&mut self) {
        info!("LibraryProvider: 正在同步统计信息...");
        let service = self.service.read().clone();
        let svc = service.lock().await;
        let stats = svc.stats();
        self.track_count.set(stats.track_count);
        self.album_count.set(stats.album_count);
        self.artist_count.set(stats.artist_count);
        self.genre_count.set(stats.genre_count);
        self.folder_count.set(stats.folder_count);
        info!(
            "LibraryProvider: 统计信息同步完成 (共 {} 首歌曲)",
            stats.track_count
        );
    }

    /// 刷新分类摘要
    pub async fn refresh_summaries(&mut self) {
        info!("LibraryProvider: 正在刷新分类摘要 (歌手/专辑/流派)...");
        let service = self.service.read().clone();
        let svc = service.lock().await;

        if let Ok(artists) = svc.get_artist_summaries() {
            self.artists.set(artists);
        }
        if let Ok(albums) = svc.get_album_summaries() {
            self.albums.set(albums);
        }
        if let Ok(genres) = svc.get_genre_summaries() {
            self.genres.set(genres);
        }
        if let Ok(folders) = svc.get_folder_summaries() {
            self.folder_summaries.set(folders);
        }
        info!("LibraryProvider: 分类摘要刷新完成");
    }

    /// 刷新音轨列表
    pub async fn refresh_tracks(&mut self) {
        let filter = self.filter.read().clone();
        info!(
            "LibraryProvider: 正在同步音轨列表... (Filter: {:?})",
            filter
        );
        let service = self.service.read().clone();
        let svc = service.lock().await;

        // 如果没有过滤条件，获取全部；否则使用过滤
        let result = if filter == TrackFilter::default() {
            svc.get_all_tracks()
        } else {
            svc.filter_tracks(filter, 1, 1000).map(|r| r.items)
        };

        if let Ok(tracks) = result {
            let count = tracks.len();
            self.all_api_tracks.set(tracks.clone());
            self.current_api_tracks.set(tracks.clone());
            self.tracks
                .set(tracks.into_iter().map(UITrack::from).collect());
            info!("LibraryProvider: 音轨列表同步完成, 共加载 {} 条记录", count);
        }
    }

    /// 设置过滤条件并刷新
    pub async fn set_filter(&mut self, filter: TrackFilter) {
        self.filter.set(filter);
        self.refresh_tracks().await;
    }

    /// 分页过滤音轨 (用于全量详情页)
    pub async fn filter_tracks_paginated(
        &self,
        filter: TrackFilter,
        page: usize,
        page_size: usize,
    ) -> Result<PaginatedResult<ApiTrack>, String> {
        let svc = self.service.read().clone();
        let s = svc.lock().await;
        s.filter_tracks(filter, page, page_size)
    }

    /// 获取完整带有歌词的单首音轨数据 (直达数据库)
    pub async fn get_track_with_lyrics(&self, id: String) -> Option<ApiTrack> {
        let svc = self.service.read().clone();
        let s = svc.lock().await;
        s.get_track_with_lyrics(&id).ok().flatten()
    }

    /// 清除过滤条件并刷新
    pub async fn clear_filter(&mut self) {
        self.set_filter(TrackFilter::default()).await;
    }

    /// 刷新文件夹列表
    pub async fn refresh_folders(&mut self) {
        info!("LibraryProvider: 正在获取扫描文件夹列表...");
        let service = self.service.read().clone();
        let svc = service.lock().await;
        if let Ok(folders) = svc.get_scan_folders() {
            let count = folders.len();
            self.folders.set(
                folders
                    .into_iter()
                    .map(|f| UIScanFolder {
                        id: f.id,
                        path: f.path,
                        track_count: 0,
                        enabled: f.enabled,
                    })
                    .collect(),
            );
            info!(
                "LibraryProvider: 扫描文件夹列表获取完成, 共 {} 个路径",
                count
            );
        }
    }

    /// 添加扫描文件夹并触发异步扫描 (非阻塞，精细锁管理)
    pub async fn add_folder(&mut self, path: String) -> Result<(), String> {
        let service = self.service.read().clone();
        let normalized_path = path.replace('\\', "/");

        // 1. 先将目录记录同步写入数据库
        {
            let mut svc = service.lock().await;
            if let Some(store) = &svc.store {
                store
                    .add_scan_folder(&normalized_path, true)
                    .map_err(|e| e.to_string())?;
                let _ = svc.refresh_stats();
            } else {
                return Err("Database not initialized".to_string());
            }
        }

        // 2. 刷新文件夹列表与统计信息 UI
        self.refresh_folders().await;
        self.refresh_stats().await;

        // 3. 在后台启动异步扫描
        let mut provider = *self;
        spawn(async move {
            provider.perform_background_scan(normalized_path).await;
        });

        Ok(())
    }

    /// 执行细粒度锁管理的后台扫描流程
    pub async fn perform_background_scan(&mut self, path: String) {
        let service = self.service.read().clone();
        let mut provider = *self;

        provider.is_scanning.set(true);
        info!("LibraryProvider: 启动极速后台异步并发扫描任务 -> {}", path);

        // A. 获取文件列表和增量表
        let (audio_files, mtime_map, cover_dir) = {
            let svc = service.lock().await;
            let files = svc.get_audio_files(&path).unwrap_or_default();
            let map = svc.store.as_ref()
                .map(|s| s.get_path_mtime_map(&path).unwrap_or_default())
                .unwrap_or_default();
            (files, map, svc.cover_dir())
        };

        let total = audio_files.len();
        if total == 0 {
            info!("LibraryProvider: 目录为空，无需扫描: {}", path);
            provider.is_scanning.set(false);
            return;
        }

        // B. 过滤出需要更新的文件
        let mut to_process = Vec::new();
        for file in audio_files {
            let file_path_str = file.path.to_string_lossy().to_string();
            let mtime = std::fs::metadata(&file.path)
                .and_then(|m| m.modified())
                .unwrap_or(std::time::UNIX_EPOCH)
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;

            if let Some(&old_mtime) = mtime_map.get(&file_path_str) {
                if old_mtime == mtime {
                    continue;
                }
            }
            to_process.push(file);
        }

        let skipped = total - to_process.len();
        info!(
            "LibraryProvider: {} 个文件未变动(增量跳过)。分发 {} 个文件至并发队列...",
            skipped,
            to_process.len()
        );

        // 提前更新已跳过部分的进度
        {
            let mut svc = service.lock().await;
            svc.scan_progress = if total > 0 { skipped as f32 / total as f32 } else { 1.0 };
        }

        if to_process.is_empty() {
            info!("LibraryProvider: 无任何变动，扫描直接完成");
            provider.refresh_all().await;
            provider.is_scanning.set(false);
            return;
        }

        // C. 并发多线程提取 (16 线程火力全开)
        let mut join_set = tokio::task::JoinSet::new();
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(16));

        for file in to_process {
            let sem = semaphore.clone();
            let c_dir = cover_dir.clone();
            join_set.spawn(async move {
                let _permit = sem.acquire().await.ok();
                api::services::library::LibraryService::extract_metadata_standalone(file, c_dir).await
            });
        }

        let mut processed_tracks = Vec::new();
        let mut processed_count = skipped;

        // D. 等待提取并批量打入数据库
        while let Some(res) = join_set.join_next().await {
            if let Ok(Some(track)) = res {
                processed_tracks.push(track);
            }

            processed_count += 1;

            // 间隙上报细粒度进度 (不写库，供前端跳动小数值)
            if processed_count % 50 == 0 && processed_tracks.len() < 500 {
                let mut svc = service.lock().await;
                svc.scan_progress = processed_count as f32 / total as f32;
            }

            // 每凑齐 500 个或者全部完成，进行一次“大兵团作战”库写入
            if processed_tracks.len() >= 500 || (processed_count == total && !processed_tracks.is_empty()) {
                let mut svc = service.lock().await;
                svc.scan_progress = processed_count as f32 / total as f32;
                
                if let Err(e) = svc.sync_tracks_batch(&processed_tracks, &path) {
                    tracing::error!("LibraryProvider: 中批次同步失败: {}", e);
                }
                
                // 释放锁并挂起等下一帧
                drop(svc);
                processed_tracks.clear();
                
                provider.refresh_stats().await;
                tokio::task::yield_now().await;
            }
        }

        info!("LibraryProvider: 并发扫描任务 {} 已极速完成", path);
        provider.refresh_all().await;
        provider.is_scanning.set(false);
    }

    /// 移除扫描文件夹
    pub async fn remove_folder(&mut self, path: String) -> Result<(), String> {
        let service = self.service.read().clone();
        let mut svc = service.lock().await;
        let result = svc.remove_scan_folder(&path);

        if result.is_ok() {
            drop(svc);
            // 立即刷新所有相关状态，确保 UI 上的音轨、统计和分类结果同步更新
            self.refresh_folders().await;
            self.refresh_stats().await;
            self.refresh_tracks().await;
            self.refresh_summaries().await;
        }
        result
    }

    /// 批量同步文件夹变更 (新增、删除、启用/禁用状态)
    pub async fn apply_folder_changes(
        &mut self,
        new_folders: Vec<UIScanFolder>,
    ) -> Result<(), String> {
        info!(
            "LibraryProvider: 准备应用文件夹变更, 目标列表: {:?}",
            new_folders
        );
        let service = self.service.read().clone();

        // 转换数据格式为后端需要的格式: (id, path, enabled)
        let target_data: Vec<(String, String, bool)> = new_folders
            .into_iter()
            .map(|f| (f.id, f.path, f.enabled))
            .collect();

        // 1. 同步执行数据库记录更新 (快速)
        let scan_paths = {
            let mut svc = service.lock().await;
            match svc.batch_sync_folders(target_data).await {
                Ok(paths) => paths,
                Err(e) => {
                    error!("LibraryProvider: 批量同步文件夹失败: {}", e);
                    return Err(e);
                }
            }
        };

        // 2. 立即刷新文件夹列表 UI
        self.refresh_folders().await;

        // 3. 为每个需要扫描的目录启动后台扫描任务
        let mut provider = *self;
        for path in scan_paths {
            spawn(async move {
                provider.perform_background_scan(path).await;
            });
        }

        // 刷新状态以确保 UI 与数据库一致
        self.refresh_all().await;
        Ok(())
    }

    /// 导航至艺术家详情
    pub async fn navigate_to_artist(&mut self, id: String) {
        info!("LibraryProvider: 正在导航至艺术家详情, ID: {}", id);
        let service = self.service.read().clone();
        let svc = service.lock().await;

        if let Ok(Some(artist)) = svc.get_artist(&id) {
            // 压入历史栈
            let current = self.current_view.read().clone();
            self.navigation_history.write().push(current);

            // 1. 获取音轨
            let mut filter = TrackFilter::default();
            filter.artist_id = Some(id.clone());

            if let Ok(tracks) = svc.filter_tracks(filter, 1, 1000) {
                self.detail_api_tracks.set(tracks.items.clone());
                self.detail_tracks
                    .set(tracks.items.into_iter().map(UITrack::from).collect());
            }

            // 2. 获取专辑
            if let Ok(albums) = svc.get_albums_by_artist(&id) {
                self.detail_albums.set(albums);
            }

            self.current_view.set(LibraryView::ArtistDetail(artist));
            self.active_global_tab.set("library".to_string());
        }
    }

    /// 导航至专辑详情
    pub async fn navigate_to_album(&mut self, id: String) {
        info!("LibraryProvider: 正在导航至专辑详情, ID: {}", id);
        let service = self.service.read().clone();
        let svc = service.lock().await;

        if let Ok(Some(album)) = svc.get_album(&id) {
            // 压入历史栈
            let current = self.current_view.read().clone();
            self.navigation_history.write().push(current);

            let mut filter = TrackFilter::default();
            filter.album_id = Some(id);

            if let Ok(tracks) = svc.filter_tracks(filter, 1, 1000) {
                self.detail_api_tracks.set(tracks.items.clone());
                self.detail_tracks
                    .set(tracks.items.into_iter().map(UITrack::from).collect());
                self.current_view.set(LibraryView::AlbumDetail(album));
                self.active_global_tab.set("library".to_string());
            }
        }
    }

    /// 导航至流派详情
    pub async fn navigate_to_genre(&mut self, id: i32) {
        info!("LibraryProvider: 正在导航至流派详情, ID: {}", id);
        let service = self.service.read().clone();
        let svc = service.lock().await;

        if let Ok(Some(genre)) = svc.get_genre(id) {
            // 压入历史栈
            let current = self.current_view.read().clone();
            self.navigation_history.write().push(current);

            // 1. 获取音轨
            let mut filter = TrackFilter::default();
            filter.genre_id = Some(id);

            if let Ok(tracks) = svc.filter_tracks(filter, 1, 1000) {
                self.detail_api_tracks.set(tracks.items.clone());
                self.detail_tracks
                    .set(tracks.items.into_iter().map(UITrack::from).collect());
                self.current_view.set(LibraryView::GenreDetail(genre));
                self.active_global_tab.set("library".to_string());
            }
        }
    }

    /// 导航至个人空间子 Tab
    pub fn navigate_to_personal_tab(&mut self, tab: String) {
        info!("LibraryProvider: 正在导航至个人空间 Tab: {}", tab);

        // 压入历史栈
        let current = self.current_view.read().clone();
        self.navigation_history.write().push(current);

        let view = match tab.as_str() {
            "overview" => LibraryView::PersonalOverview,
            "liked" => LibraryView::PersonalLiked,
            "playlists" => LibraryView::PersonalPlaylists,
            "playlists_create" => LibraryView::PersonalPlaylistsCreate,
            "albums" => LibraryView::PersonalAlbums,
            "artists" => LibraryView::PersonalArtists,
            "recent" => LibraryView::PersonalRecent,
            _ => LibraryView::PersonalOverview,
        };

        self.active_global_tab.set("personal".to_string());
        self.current_view.set(view);
    }

    /// 导航至歌单详情
    pub fn navigate_to_playlist(&mut self, playlist: ApiPlaylist) {
        info!("LibraryProvider: 正在导航至歌单详情: {}", playlist.name);

        // 压入历史栈
        let current = self.current_view.read().clone();
        self.navigation_history.write().push(current);

        self.active_global_tab.set("personal".to_string());
        self.current_view.set(LibraryView::PlaylistDetail(playlist));
    }

    /// 导航至全量歌曲详情页 (基于通用的 TrackFilter)
    pub fn navigate_to_all_tracks_detail(
        &mut self,
        title: String,
        filter: TrackFilter,
        source_type: String,
    ) {
        info!(
            "LibraryProvider: 正在导航至全量歌曲详情页: {}, 来源: {}",
            title, source_type
        );
        let current = self.current_view.read().clone();
        self.navigation_history.write().push(current);

        self.current_view.set(LibraryView::AllTracksDetail {
            title,
            filter,
            source_type,
        });
    }

    /// 导航至“我喜欢的音乐”全量详情页
    pub fn navigate_to_liked_all_detail(&mut self) {
        info!("LibraryProvider: 正在导航至“我喜欢的音乐”全量页");
        let current = self.current_view.read().clone();
        self.navigation_history.write().push(current);

        self.current_view.set(LibraryView::AllTracksDetail {
            title: "我喜欢的音乐".to_string(),
            filter: TrackFilter::default(), // 这里后面会在组件内特殊处理
            source_type: "liked".to_string(),
        });
    }

    /// 导航至“歌单”全量详情页
    pub fn navigate_to_playlist_all_detail(&mut self, playlist: ApiPlaylist) {
        info!("LibraryProvider: 正在导航至歌单“{}”全量页", playlist.name);
        let current = self.current_view.read().clone();
        self.navigation_history.write().push(current);

        self.current_view.set(LibraryView::AllTracksDetail {
            title: playlist.name.clone(),
            filter: TrackFilter::default(), // 后面通过 source_id 处理
            source_type: format!("playlist:{}", playlist.id),
        });
    }

    /// 返回上一级视图 (通过历史栈)
    pub fn navigate_back(&mut self) {
        info!("LibraryProvider: 正在执行逻辑后退...");
        let mut history = self.navigation_history.write();
        if let Some(prev_view) = history.pop() {
            // 同步全局 Tab 状态
            let new_tab = match &prev_view {
                LibraryView::Main
                | LibraryView::ArtistDetail(_)
                | LibraryView::AlbumDetail(_)
                | LibraryView::GenreDetail(_)
                | LibraryView::AllTracksDetail { .. } => "library",
                LibraryView::PersonalOverview
                | LibraryView::PersonalLiked
                | LibraryView::PersonalPlaylists
                | LibraryView::PersonalPlaylistsCreate
                | LibraryView::PlaylistDetail(_)
                | LibraryView::PersonalAlbums
                | LibraryView::PersonalArtists
                | LibraryView::PersonalRecent => "personal",
            };
            self.active_global_tab.set(new_tab.to_string());
            self.current_view.set(prev_view);
        } else {
            info!("LibraryProvider: 历史栈已空，返回主视图");
            self.active_global_tab.set("library".to_string());
            self.current_view.set(LibraryView::Main);
        }
    }
}
