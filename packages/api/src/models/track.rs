#[derive(Clone, PartialEq, Debug)]
pub struct Track {
    pub id: String,             // 文件哈希 (SHA-256)
    pub path: String,           // 物理路径
    pub title: String,          // 歌名
    pub artist: String,         // 艺术家名称 (冗余用于快速展示)
    pub artist_id: String,      // 关联艺术家 ID
    pub album: String,          // 专辑名称 (冗余用于快速展示)
    pub album_id: String,       // 关联专辑 ID
    pub duration: u32,          // 时长(秒)
    pub size: u64,              // 文件大小
    pub bitrate: Option<u32>,   // 比特率
    pub extension: String,      // 后缀名
    pub genres: Vec<String>,    // 流派列表 (支持多个)
    pub added_at: i64,          // 添加时间
    pub mtime: i64,             // 文件最后修改时间
    pub cover: Option<String>,  // 封面缓存路径
    pub lyrics: Option<String>, // 歌词内容
    pub played_at: Option<i64>, // 最近播放时间（如果是播放历史）
}

#[derive(Clone, PartialEq, Debug)]
pub struct LibraryStats {
    pub track_count: usize,
    pub album_count: usize,
    pub artist_count: usize,
    pub genre_count: usize,
    pub folder_count: usize,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Album {
    pub id: String,
    pub title: String,
    pub artist_id: String,   // 关联艺术家 ID
    pub artist_name: String, // 艺术家名称 (方便 UI 显示)
    pub cover_path: Option<String>,
    pub description: Option<String>, // 专辑简介
    pub track_count: i32,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Artist {
    pub id: String, // 艺术家 ID
    pub name: String,
    pub cover_path: Option<String>, // 艺术家封面/头像
    pub bio: Option<String>,        // 艺术家介绍
    pub track_count: i32,
}

#[derive(Clone, PartialEq, Debug)]
pub struct GenreSummary {
    pub id: i32,
    pub name: String,
    pub image: Option<String>,
    pub description: Option<String>,
    pub track_count: i32,
}

#[derive(Clone, PartialEq, Debug)]
pub struct FolderSummary {
    pub path: String,
    pub track_count: i32,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ScanFolder {
    pub id: String,
    pub path: String,
    pub enabled: bool,
    pub last_scan_at: Option<i64>,
}
#[derive(Clone, PartialEq, Debug, Default)]
pub struct TrackFilter {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub artist_id: Option<String>,
    pub album: Option<String>,
    pub album_id: Option<String>,
    pub genres: Option<Vec<String>>,
    pub genre_id: Option<i32>,
    pub folder_path_prefix: Option<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
}
