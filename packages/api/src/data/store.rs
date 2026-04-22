use crate::models::{
    Album, Artist, FolderSummary, GenreSummary, LikedAlbum, PlayHistory, ScanFolder, Track,
    UserPlaylist,
};
use rusqlite::{params, Connection, OptionalExtension, Result};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn new(db_path: PathBuf) -> Result<Self, StoreError> {
        let conn = Connection::open(db_path)?;
        Self::init_tables(&conn)?;
        Ok(Store { conn })
    }

    fn init_tables(conn: &Connection) -> Result<(), StoreError> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tracks (
                id TEXT PRIMARY KEY,
                path TEXT UNIQUE,
                title TEXT,
                artist TEXT,
                artist_id TEXT,
                album TEXT,
                album_id TEXT,
                duration INTEGER,
                size INTEGER,
                bitrate INTEGER,
                extension TEXT,
                genre TEXT,
                added_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                mtime INTEGER,
                cover TEXT,
                lyrics TEXT,
                FOREIGN KEY (artist_id) REFERENCES artists(id),
                FOREIGN KEY (album_id) REFERENCES albums(id)
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS genres (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE,
                image TEXT,
                description TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS track_genres (
                track_id TEXT,
                genre_id INTEGER,
                PRIMARY KEY (track_id, genre_id),
                FOREIGN KEY (track_id) REFERENCES tracks(id) ON DELETE CASCADE,
                FOREIGN KEY (genre_id) REFERENCES genres(id) ON DELETE CASCADE
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS artists (
                id TEXT PRIMARY KEY,
                name TEXT UNIQUE,
                cover_path TEXT,
                bio TEXT,
                track_count INTEGER DEFAULT 0
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS albums (
                id TEXT PRIMARY KEY,
                title TEXT,
                artist_id TEXT,
                cover_path TEXT,
                description TEXT,
                track_count INTEGER DEFAULT 0,
                FOREIGN KEY (artist_id) REFERENCES artists(id)
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS scan_folders (
                id TEXT PRIMARY KEY,
                path TEXT UNIQUE,
                enabled BOOLEAN DEFAULT 1,
                last_scan_at INTEGER
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_track_title ON tracks(title)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_track_artist ON tracks(artist)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_track_path ON tracks(path)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_track_artist_id ON tracks(artist_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_track_album_id ON tracks(album_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_track_genres_genre_id ON track_genres(genre_id)",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS playlists (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT DEFAULT '',
                cover TEXT,
                track_count INTEGER DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                id TEXT PRIMARY KEY,
                name TEXT UNIQUE
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS playlist_tags (
                playlist_id TEXT,
                tag_id TEXT,
                PRIMARY KEY (playlist_id, tag_id),
                FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS playlist_tracks (
                playlist_id TEXT NOT NULL,
                track_id TEXT NOT NULL,
                position INTEGER NOT NULL,
                added_at INTEGER NOT NULL,
                PRIMARY KEY (playlist_id, track_id),
                FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE,
                FOREIGN KEY (track_id) REFERENCES tracks(id) ON DELETE CASCADE
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS liked_tracks (
                track_id TEXT PRIMARY KEY,
                liked_at INTEGER NOT NULL,
                FOREIGN KEY (track_id) REFERENCES tracks(id) ON DELETE CASCADE
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS liked_albums (
                album_id TEXT PRIMARY KEY,
                album_title TEXT NOT NULL,
                album_artist TEXT NOT NULL,
                album_cover TEXT,
                liked_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS liked_artists (
                artist_name TEXT PRIMARY KEY,
                liked_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS play_history (
                id TEXT PRIMARY KEY,
                track_id TEXT NOT NULL,
                played_at INTEGER NOT NULL,
                FOREIGN KEY (track_id) REFERENCES tracks(id) ON DELETE CASCADE
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_playlist_tracks_position ON playlist_tracks(playlist_id, position)",
            [],
        )?;

        Ok(())
    }

    pub fn insert_track(&self, track: &Track) -> Result<(), StoreError> {
        self.insert_track_batch(std::slice::from_ref(track))
    }

    pub fn insert_track_batch(&self, tracks: &[Track]) -> Result<(), StoreError> {
        let tx = self.conn.unchecked_transaction()?;
        for track in tracks {
            // 1. 确保艺术家存在
            tx.execute(
                "INSERT OR IGNORE INTO artists (id, name) VALUES (?1, ?2)",
                params![track.artist_id, track.artist],
            )?;

            // 2. 确保专辑存在
            tx.execute(
                "INSERT OR IGNORE INTO albums (id, title, artist_id, cover_path) VALUES (?1, ?2, ?3, ?4)",
                params![track.album_id, track.album, track.artist_id, track.cover],
            )?;

            // 3. 插入或替换音轨
            tx.execute(
                "INSERT OR REPLACE INTO tracks 
                 (id, path, title, artist, artist_id, album, album_id, duration, size, bitrate, extension, added_at, mtime, cover, lyrics)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
                params![
                    track.id,
                    track.path,
                    track.title,
                    track.artist,
                    track.artist_id,
                    track.album,
                    track.album_id,
                    track.duration as i64,
                    track.size as i64,
                    track.bitrate.map(|b| b as i64),
                    track.extension,
                    track.added_at,
                    track.mtime,
                    track.cover,
                    track.lyrics,
                ],
            )?;

            // 处理流派多对多关系
            for genre_name in &track.genres {
                // 1. 确保流派存在
                tx.execute(
                    "INSERT OR IGNORE INTO genres (name) VALUES (?1)",
                    params![genre_name],
                )?;

                // 2. 获取 ID (因为是自增 ID，需要根据名称查询)
                let genre_id: i64 = tx.query_row(
                    "SELECT id FROM genres WHERE name = ?1",
                    params![genre_name],
                    |row| row.get(0),
                )?;

                // 3. 建立关联
                tx.execute(
                    "INSERT OR IGNORE INTO track_genres (track_id, genre_id) VALUES (?1, ?2)",
                    params![track.id, genre_id],
                )?;
            }
        }

        // --- 扫描完成后进行元数据与统计校准 ---
        // 1. 自动补全封面缺失的艺术家
        tx.execute(
            "UPDATE artists SET cover_path = (
                SELECT cover FROM tracks 
                WHERE artist_id = artists.id AND cover IS NOT NULL 
                LIMIT 1
             ) WHERE cover_path IS NULL",
            [],
        )?;

        // 2. 校准艺术家曲目总数
        tx.execute(
            "UPDATE artists SET track_count = (
                SELECT COUNT(*) FROM tracks WHERE artist_id = artists.id
             )",
            [],
        )?;

        // 3. 校准专辑曲目总数
        tx.execute(
            "UPDATE albums SET track_count = (
                SELECT COUNT(*) FROM tracks WHERE album_id = albums.id
             )",
            [],
        )?;

        tx.commit()?;
        Ok(())
    }

    pub fn get_all_tracks(&self) -> Result<Vec<Track>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.path, t.title, t.artist, t.artist_id, t.album, t.album_id, t.duration, t.size, t.bitrate, t.extension, 
                    (SELECT GROUP_CONCAT(g.name, ';') FROM track_genres tg JOIN genres g ON tg.genre_id = g.id WHERE tg.track_id = t.id) as genres,
                    t.added_at, t.mtime, t.cover, NULL as lyrics
             FROM tracks t ORDER BY t.added_at DESC"
        )?;
        let tracks = stmt.query_map([], |row| {
            let genres_raw: Option<String> = row.get(11)?;
            let genres = genres_raw
                .map(|s| s.split(';').map(|g| g.to_string()).collect())
                .unwrap_or_default();

            Ok(Track {
                id: row.get(0)?,
                path: row.get(1)?,
                title: row.get(2)?,
                artist: row.get(3)?,
                artist_id: row.get(4)?,
                album: row.get(5)?,
                album_id: row.get(6)?,
                duration: row.get(7)?,
                size: row.get(8)?,
                bitrate: row.get(9)?,
                extension: row.get(10)?,
                genres,
                added_at: row.get(12)?,
                mtime: row.get(13)?,
                cover: row.get(14)?,
                lyrics: row.get(15)?,
                played_at: None,
            })
        })?;
        tracks
            .collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::from)
    }

    pub fn get_track(&self, id: &str) -> Result<Option<Track>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.path, t.title, t.artist, t.artist_id, t.album, t.album_id, t.duration, t.size, t.bitrate, t.extension, 
                    (SELECT GROUP_CONCAT(g.name, ';') FROM track_genres tg JOIN genres g ON tg.genre_id = g.id WHERE tg.track_id = t.id) as genres,
                    t.added_at, t.mtime, t.cover, t.lyrics
             FROM tracks t WHERE t.id = ?1"
        )?;
        let track = stmt
            .query_row(params![id], |row| {
                let genres_raw: Option<String> = row.get(11)?;
                let genres = genres_raw
                    .map(|s| s.split(';').map(|g| g.to_string()).collect())
                    .unwrap_or_default();

                Ok(Track {
                    id: row.get(0)?,
                    path: row.get(1)?,
                    title: row.get(2)?,
                    artist: row.get(3)?,
                    artist_id: row.get(4)?,
                    album: row.get(5)?,
                    album_id: row.get(6)?,
                    duration: row.get(7)?,
                    size: row.get(8)?,
                    bitrate: row.get(9)?,
                    extension: row.get(10)?,
                    genres,
                    added_at: row.get(12)?,
                    mtime: row.get(13)?,
                    cover: row.get(14)?,
                    lyrics: row.get(15)?,
                    played_at: None,
                })
            })
            .optional()?;
        Ok(track)
    }

    pub fn delete_track(&self, id: &str) -> Result<(), StoreError> {
        self.conn
            .execute("DELETE FROM tracks WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_track_count(&self) -> Result<usize, StoreError> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM tracks", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    pub fn get_album_count(&self) -> Result<usize, StoreError> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(DISTINCT album) FROM tracks WHERE album IS NOT NULL",
            [],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    pub fn get_artist_count(&self) -> Result<usize, StoreError> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(DISTINCT artist) FROM tracks WHERE artist IS NOT NULL",
            [],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    pub fn get_genre_count(&self) -> Result<usize, StoreError> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM genres", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    pub fn get_scan_folder_count(&self) -> Result<usize, StoreError> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM scan_folders WHERE enabled = 1",
            [],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    pub fn add_scan_folder(&self, path: &str, enabled: bool) -> Result<(), StoreError> {
        let id = Uuid::new_v4().to_string();
        self.conn.execute(
            "INSERT INTO scan_folders (id, path, enabled) VALUES (?1, ?2, ?3)
             ON CONFLICT(path) DO UPDATE SET enabled = ?3",
            params![id, path, enabled],
        )?;
        Ok(())
    }

    /// 更新扫描目录的启用状态
    pub fn update_scan_folder_enabled(&self, id: &str, enabled: bool) -> Result<(), StoreError> {
        self.conn.execute(
            "UPDATE scan_folders SET enabled = ?1 WHERE id = ?2",
            params![enabled, id],
        )?;
        Ok(())
    }

    pub fn remove_scan_folder(&self, path: &str) -> Result<(), StoreError> {
        self.conn
            .execute("DELETE FROM scan_folders WHERE path = ?1", params![path])?;
        Ok(())
    }

    /// 根据路径前缀物理删除数据库中的所有音轨记录。
    ///
    /// # 参数
    /// * `prefix` - 文件夹路径前缀，例如 "C:/Music"
    ///
    /// # 示例
    /// ```rust
    /// store.delete_tracks_by_path_prefix("C:/Music")?;
    /// ```
    pub fn delete_tracks_by_path_prefix(&self, prefix: &str) -> Result<(), StoreError> {
        let pattern = format!("{}%", prefix);
        self.conn
            .execute("DELETE FROM tracks WHERE path LIKE ?1", params![pattern])?;
        Ok(())
    }

    /// 更新特定扫描文件夹的最后一次扫描时间戳。
    ///
    /// # 参数
    /// * `path` - 文件夹路径
    /// * `timestamp` - Unix 时间戳
    ///
    /// # 示例
    /// ```rust
    /// store.update_folder_scan_time("C:/Music", 1625097600)?;
    /// ```
    pub fn update_folder_scan_time(&self, path: &str, timestamp: i64) -> Result<(), StoreError> {
        self.conn.execute(
            "UPDATE scan_folders SET last_scan_at = ?2 WHERE path = ?1",
            params![path, timestamp],
        )?;
        Ok(())
    }

    pub fn get_scan_folders(&self) -> Result<Vec<ScanFolder>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, path, enabled, last_scan_at FROM scan_folders")?;
        let folders = stmt.query_map([], |row| {
            Ok(ScanFolder {
                id: row.get(0)?,
                path: row.get(1)?,
                enabled: row.get(2)?,
                last_scan_at: row.get(3)?,
            })
        })?;
        folders
            .collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::from)
    }

    /// 关键字模糊搜索音轨（匹配歌名或歌手），支持分页
    pub fn search_tracks(
        &self,
        keyword: &str,
        limit: usize,
        offset: usize,
    ) -> Result<(Vec<Track>, usize), StoreError> {
        let pattern = format!("%{}%", keyword);

        // 1. 获取总数
        let total: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM tracks WHERE title LIKE ?1 OR artist LIKE ?1",
            params![pattern],
            |row| row.get(0),
        )?;

        // 2. 获取当前页数据
        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.path, t.title, t.artist, t.artist_id, t.album, t.album_id, t.duration, t.size, t.bitrate, t.extension, 
                    (SELECT GROUP_CONCAT(g.name, ';') FROM track_genres tg JOIN genres g ON tg.genre_id = g.id WHERE tg.track_id = t.id) as genres,
                    t.added_at, t.mtime, t.cover, NULL as lyrics
             FROM tracks t
             WHERE t.title LIKE ?1 OR t.artist LIKE ?1 
             ORDER BY t.added_at DESC LIMIT ?2 OFFSET ?3"
        )?;

        let tracks = stmt
            .query_map(params![pattern, limit as i64, offset as i64], |row| {
                let genres_raw: Option<String> = row.get(11)?;
                let genres = genres_raw
                    .map(|s| s.split(';').map(|g| g.to_string()).collect())
                    .unwrap_or_default();

                Ok(Track {
                    id: row.get(0)?,
                    path: row.get(1)?,
                    title: row.get(2)?,
                    artist: row.get(3)?,
                    artist_id: row.get(4)?,
                    album: row.get(5)?,
                    album_id: row.get(6)?,
                    duration: row.get(7)?,
                    size: row.get(8)?,
                    bitrate: row.get(9)?,
                    extension: row.get(10)?,
                    genres,
                    added_at: row.get(12)?,
                    mtime: row.get(13)?,
                    cover: row.get(14)?,
                    lyrics: row.get(15)?,
                    played_at: None,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok((tracks, total as usize))
    }

    /// 多条件过滤音轨，支持分页
    pub fn filter_tracks(
        &self,
        filter: crate::models::TrackFilter,
        limit: usize,
        offset: usize,
    ) -> Result<(Vec<Track>, usize), StoreError> {
        let mut query = "SELECT t.id, t.path, t.title, t.artist, t.artist_id, t.album, t.album_id, t.duration, t.size, t.bitrate, t.extension, (SELECT GROUP_CONCAT(g.name, ';') FROM track_genres tg JOIN genres g ON tg.genre_id = g.id WHERE tg.track_id = t.id) as genres, t.added_at, t.mtime, t.cover, NULL as lyrics FROM tracks t WHERE 1=1".to_string();
        let mut count_query = "SELECT COUNT(*) FROM tracks WHERE 1=1".to_string();
        let mut sql_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(artist) = &filter.artist {
            query.push_str(" AND artist = ?");
            count_query.push_str(" AND artist = ?");
            sql_params.push(Box::new(artist.clone()));
        }
        if let Some(artist_id) = &filter.artist_id {
            query.push_str(" AND artist_id = ?");
            count_query.push_str(" AND artist_id = ?");
            sql_params.push(Box::new(artist_id.clone()));
        }
        if let Some(album) = &filter.album {
            query.push_str(" AND album = ?");
            count_query.push_str(" AND album = ?");
            sql_params.push(Box::new(album.clone()));
        }
        if let Some(album_id) = &filter.album_id {
            query.push_str(" AND album_id = ?");
            count_query.push_str(" AND album_id = ?");
            sql_params.push(Box::new(album_id.clone()));
        }
        if let Some(genre_id) = &filter.genre_id {
            let sub_query =
                "EXISTS (SELECT 1 FROM track_genres WHERE track_id = t.id AND genre_id = ?)";
            query.push_str(&format!(" AND {}", sub_query));
            count_query.push_str(" AND EXISTS (SELECT 1 FROM track_genres tg WHERE tg.track_id = tracks.id AND tg.genre_id = ?)");
            sql_params.push(Box::new(*genre_id));
        }
        if let Some(genres) = &filter.genres {
            if !genres.is_empty() {
                let place_holders: String =
                    genres.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                let sub_query = format!("EXISTS (SELECT 1 FROM track_genres tg JOIN genres g ON tg.genre_id = g.id WHERE tg.track_id = t.id AND g.name IN ({}))", place_holders);
                query.push_str(&format!(" AND {}", sub_query));
                count_query.push_str(&format!(" AND EXISTS (SELECT 1 FROM track_genres tg JOIN genres g ON tg.genre_id = g.id WHERE tg.track_id = tracks.id AND g.name IN ({}))", place_holders)); // Note: count_query here uses 'tracks' table directly
                for genre in genres {
                    sql_params.push(Box::new(genre.clone()));
                }
            }
        }
        if let Some(prefix) = &filter.folder_path_prefix {
            query.push_str(" AND path LIKE ?");
            count_query.push_str(" AND path LIKE ?");
            sql_params.push(Box::new(format!("{}%", prefix)));
        }

        // 获取总数
        let mut count_stmt = self.conn.prepare(&count_query)?;
        let total: i64 =
            count_stmt.query_row(rusqlite::params_from_iter(&sql_params), |row| row.get(0))?;

        // 获取分页数据
        query.push_str(" ORDER BY added_at DESC LIMIT ? OFFSET ?");
        sql_params.push(Box::new(limit as i64));
        sql_params.push(Box::new(offset as i64));

        let mut stmt = self.conn.prepare(&query)?;
        let tracks = stmt
            .query_map(rusqlite::params_from_iter(&sql_params), |row| {
                let genres_raw: Option<String> = row.get(11)?;
                let genres = genres_raw
                    .map(|s| s.split(';').map(|g| g.to_string()).collect())
                    .unwrap_or_default();

                Ok(Track {
                    id: row.get(0)?,
                    path: row.get(1)?,
                    title: row.get(2)?,
                    artist: row.get(3)?,
                    artist_id: row.get(4)?,
                    album: row.get(5)?,
                    album_id: row.get(6)?,
                    duration: row.get(7)?,
                    size: row.get(8)?,
                    bitrate: row.get(9)?,
                    extension: row.get(10)?,
                    genres,
                    added_at: row.get(12)?,
                    mtime: row.get(13)?,
                    cover: row.get(14)?,
                    lyrics: row.get(15)?,
                    played_at: None,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok((tracks, total as usize))
    }

    /// 获取库中所有唯一的专辑名称
    pub fn get_unique_albums(&self) -> Result<Vec<String>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT album FROM tracks WHERE album IS NOT NULL ORDER BY album")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        rows.collect::<Result<Vec<String>, _>>()
            .map_err(StoreError::from)
    }

    /// 获取库中所有唯一的艺术家名称
    pub fn get_unique_artists(&self) -> Result<Vec<String>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT artist FROM tracks WHERE artist IS NOT NULL ORDER BY artist",
        )?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        rows.collect::<Result<Vec<String>, _>>()
            .map_err(StoreError::from)
    }

    /// 获取库中所有唯一的流派名称
    pub fn get_unique_genres(&self) -> Result<Vec<String>, StoreError> {
        let mut stmt = self.conn.prepare("SELECT name FROM genres ORDER BY name")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        rows.collect::<Result<Vec<String>, _>>()
            .map_err(StoreError::from)
    }

    pub fn get_albums_by_artist(&self, artist_id: &str) -> Result<Vec<Album>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT a.id, a.title, a.artist_id, ar.name as artist_name, a.cover_path, a.description, a.track_count 
             FROM albums a 
             LEFT JOIN artists ar ON a.artist_id = ar.id 
             WHERE a.artist_id = ?1
             ORDER BY a.title",
        )?;
        let rows = stmt.query_map(params![artist_id], |row| {
            Ok(Album {
                id: row.get(0)?,
                title: row.get(1)?,
                artist_id: row.get(2)?,
                artist_name: row.get(3).unwrap_or_else(|_| "未知艺术家".to_string()),
                cover_path: row.get(4)?,
                description: row.get(5)?,
                track_count: row.get(6)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::from)
    }

    pub fn get_artist_summaries(&self) -> Result<Vec<Artist>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, cover_path, bio, track_count FROM artists ORDER BY name")?;
        let rows = stmt.query_map([], |row| {
            Ok(Artist {
                id: row.get(0)?,
                name: row.get(1)?,
                cover_path: row.get(2)?,
                bio: row.get(3)?,
                track_count: row.get(4)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::from)
    }

    pub fn get_path_mtime_map(
        &self,
        path_prefix: &str,
    ) -> Result<std::collections::HashMap<String, i64>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT path, mtime FROM tracks WHERE path LIKE ?")?;
        let rows = stmt.query_map([format!("{}%", path_prefix)], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;

        let mut map = std::collections::HashMap::new();
        for row in rows {
            let (path, mtime) = row?;
            map.insert(path, mtime);
        }
        Ok(map)
    }

    pub fn get_album_summaries(&self) -> Result<Vec<Album>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT a.id, a.title, a.artist_id, ar.name as artist_name, a.cover_path, a.description, a.track_count 
             FROM albums a 
             LEFT JOIN artists ar ON a.artist_id = ar.id 
             ORDER BY a.title",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Album {
                id: row.get(0)?,
                title: row.get(1)?,
                artist_id: row.get(2)?,
                artist_name: row.get(3).unwrap_or_else(|_| "未知艺术家".to_string()),
                cover_path: row.get(4)?,
                description: row.get(5)?,
                track_count: row.get(6)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::from)
    }

    pub fn get_genre_summaries(&self) -> Result<Vec<GenreSummary>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT g.id, g.name, g.image, g.description, COUNT(tg.track_id) 
             FROM genres g 
             LEFT JOIN track_genres tg ON g.id = tg.genre_id 
             GROUP BY g.id, g.name, g.image, g.description 
             ORDER BY g.name",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(GenreSummary {
                id: row.get(0)?,
                name: row.get(1)?,
                image: row.get(2)?,
                description: row.get(3)?,
                track_count: row.get(4)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::from)
    }

    pub fn get_folder_summaries(&self) -> Result<Vec<FolderSummary>, StoreError> {
        let mut stmt = self.conn.prepare("SELECT path FROM tracks")?;
        let paths: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;

        let mut counts = std::collections::HashMap::new();
        for path in paths {
            if let Some(parent) = std::path::Path::new(&path).parent() {
                let parent_str = parent.to_string_lossy().to_string();
                *counts.entry(parent_str).or_insert(0) += 1;
            }
        }

        let mut summaries: Vec<_> = counts
            .into_iter()
            .map(|(path, count)| FolderSummary {
                path,
                track_count: count,
            })
            .collect();
        summaries.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(summaries)
    }

    pub fn get_artist(&self, id: &str) -> Result<Option<Artist>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, cover_path, bio, track_count FROM artists WHERE id = ?1")?;
        let artist = stmt
            .query_row(params![id], |row| {
                Ok(Artist {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    cover_path: row.get(2)?,
                    bio: row.get(3)?,
                    track_count: row.get(4)?,
                })
            })
            .optional()?;
        Ok(artist)
    }

    pub fn get_album(&self, id: &str) -> Result<Option<Album>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT a.id, a.title, a.artist_id, ar.name as artist_name, a.cover_path, a.description, a.track_count 
             FROM albums a 
             LEFT JOIN artists ar ON a.artist_id = ar.id 
             WHERE a.id = ?1",
        )?;
        let album = stmt
            .query_row(params![id], |row| {
                Ok(Album {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    artist_id: row.get(2)?,
                    artist_name: row.get(3).unwrap_or_else(|_| "未知艺术家".to_string()),
                    cover_path: row.get(4)?,
                    description: row.get(5)?,
                    track_count: row.get(6)?,
                })
            })
            .optional()?;
        Ok(album)
    }

    pub fn get_genre(&self, id: i32) -> Result<Option<GenreSummary>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT g.id, g.name, g.image, g.description, COUNT(tg.track_id) 
             FROM genres g 
             LEFT JOIN track_genres tg ON g.id = tg.genre_id 
             WHERE g.id = ?1
             GROUP BY g.id, g.name, g.image, g.description",
        )?;
        let genre = stmt
            .query_row(params![id], |row| {
                Ok(GenreSummary {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    image: row.get(2)?,
                    description: row.get(3)?,
                    track_count: row.get(4)?,
                })
            })
            .optional()?;
        Ok(genre)
    }

    pub fn like_track(&self, track_id: &str) -> Result<(), StoreError> {
        let now = chrono::Utc::now().timestamp();
        self.conn.execute(
            "INSERT OR IGNORE INTO liked_tracks (track_id, liked_at) VALUES (?1, ?2)",
            params![track_id, now],
        )?;
        Ok(())
    }

    pub fn unlike_track(&self, track_id: &str) -> Result<(), StoreError> {
        self.conn.execute(
            "DELETE FROM liked_tracks WHERE track_id = ?1",
            params![track_id],
        )?;
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

    pub fn get_liked_track_count(&self) -> Result<usize, StoreError> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM liked_tracks", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    pub fn get_liked_tracks_paginated(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<(Vec<Track>, usize), StoreError> {
        let total = self.get_liked_track_count()?;

        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.path, t.title, t.artist, t.artist_id, t.album, t.album_id, t.duration, t.size, t.bitrate, t.extension, 
                    (SELECT GROUP_CONCAT(g.name, ';') FROM track_genres tg JOIN genres g ON tg.genre_id = g.id WHERE tg.track_id = t.id) as genres,
                    t.added_at, t.mtime, t.cover, t.lyrics
             FROM tracks t
             INNER JOIN liked_tracks lt ON t.id = lt.track_id
             ORDER BY lt.liked_at DESC LIMIT ?1 OFFSET ?2"
        )?;

        let tracks = stmt
            .query_map(params![limit as i64, offset as i64], |row| {
                let genres_raw: Option<String> = row.get(11)?;
                let genres = genres_raw
                    .map(|s| s.split(';').map(|g| g.to_string()).collect())
                    .unwrap_or_default();
                Ok(Track {
                    id: row.get(0)?,
                    path: row.get(1)?,
                    title: row.get(2)?,
                    artist: row.get(3)?,
                    artist_id: row.get(4)?,
                    album: row.get(5)?,
                    album_id: row.get(6)?,
                    duration: row.get(7)?,
                    size: row.get(8)?,
                    bitrate: row.get(9)?,
                    extension: row.get(10)?,
                    genres,
                    added_at: row.get(12)?,
                    mtime: row.get(13)?,
                    cover: row.get(14)?,
                    lyrics: row.get(15)?,
                    played_at: None,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok((tracks, total))
    }

    pub fn get_liked_track_ids(&self) -> Result<Vec<String>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT track_id FROM liked_tracks ORDER BY liked_at DESC")?;
        let ids = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;
        Ok(ids)
    }

    pub fn insert_playlist(&self, playlist: &UserPlaylist) -> Result<(), StoreError> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
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

        for tag_name in &playlist.tags {
            tx.execute(
                "INSERT OR IGNORE INTO tags (id, name) VALUES (?1, ?1)",
                params![tag_name],
            )?;
            tx.execute(
                "INSERT OR IGNORE INTO playlist_tags (playlist_id, tag_id) VALUES (?1, ?2)",
                params![playlist.id, tag_name],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn get_all_playlists(&self) -> Result<Vec<UserPlaylist>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, cover, track_count, created_at, updated_at
             FROM playlists ORDER BY updated_at DESC",
        )?;
        let playlist_rows: Vec<(String, String, String, Option<String>, i64, i64, i64)> = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let description: String = row.get(2)?;
                let cover: Option<String> = row.get(3)?;
                let track_count: i64 = row.get(4)?;
                let created_at: i64 = row.get(5)?;
                let updated_at: i64 = row.get(6)?;
                Ok((
                    id,
                    name,
                    description,
                    cover,
                    track_count,
                    created_at,
                    updated_at,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let mut playlists = Vec::new();
        for (id, name, description, cover, track_count, created_at, updated_at) in playlist_rows {
            let track_ids = self.get_playlist_track_ids(&id)?;
            let tags = self.get_playlist_tags(&id)?;
            playlists.push(UserPlaylist {
                id,
                name,
                description,
                tags,
                cover,
                track_ids,
                track_count: track_count as usize,
                created_at,
                updated_at,
            });
        }
        Ok(playlists)
    }

    pub fn get_playlist(&self, id: &str) -> Result<Option<UserPlaylist>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, cover, track_count, created_at, updated_at
             FROM playlists WHERE id = ?1",
        )?;
        let row_result = stmt
            .query_row(params![id], |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let description: String = row.get(2)?;
                let cover: Option<String> = row.get(3)?;
                let track_count: i64 = row.get(4)?;
                let created_at: i64 = row.get(5)?;
                let updated_at: i64 = row.get(6)?;
                Ok((
                    id,
                    name,
                    description,
                    cover,
                    track_count,
                    created_at,
                    updated_at,
                ))
            })
            .optional()?;

        if let Some((id, name, description, cover, track_count, created_at, updated_at)) =
            row_result
        {
            let track_ids = self.get_playlist_track_ids(&id)?;
            let tags = self.get_playlist_tags(&id)?;
            Ok(Some(UserPlaylist {
                id,
                name,
                description,
                tags,
                cover,
                track_ids,
                track_count: track_count as usize,
                created_at,
                updated_at,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_playlist_track_count(&self, playlist_id: &str) -> Result<usize, StoreError> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM playlist_tracks WHERE playlist_id = ?1",
            params![playlist_id],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    pub fn get_playlist_tracks_paginated(
        &self,
        playlist_id: &str,
        limit: usize,
        offset: usize,
    ) -> Result<(Vec<Track>, usize), StoreError> {
        let total = self.get_playlist_track_count(playlist_id)?;

        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.path, t.title, t.artist, t.artist_id, t.album, t.album_id, t.duration, t.size, t.bitrate, t.extension, 
                    (SELECT GROUP_CONCAT(g.name, ';') FROM track_genres tg JOIN genres g ON tg.genre_id = g.id WHERE tg.track_id = t.id) as genres,
                    t.added_at, t.mtime, t.cover, t.lyrics
             FROM tracks t
             INNER JOIN playlist_tracks pt ON t.id = pt.track_id
             WHERE pt.playlist_id = ?1
             ORDER BY pt.position ASC LIMIT ?2 OFFSET ?3"
        )?;

        let tracks = stmt
            .query_map(params![playlist_id, limit as i64, offset as i64], |row| {
                let genres_raw: Option<String> = row.get(11)?;
                let genres = genres_raw
                    .map(|s| s.split(';').map(|g| g.to_string()).collect())
                    .unwrap_or_default();
                Ok(Track {
                    id: row.get(0)?,
                    path: row.get(1)?,
                    title: row.get(2)?,
                    artist: row.get(3)?,
                    artist_id: row.get(4)?,
                    album: row.get(5)?,
                    album_id: row.get(6)?,
                    duration: row.get(7)?,
                    size: row.get(8)?,
                    bitrate: row.get(9)?,
                    extension: row.get(10)?,
                    genres,
                    added_at: row.get(12)?,
                    mtime: row.get(13)?,
                    cover: row.get(14)?,
                    lyrics: row.get(15)?,
                    played_at: None,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok((tracks, total))
    }

    pub fn get_playlist_tracks(&self, playlist_id: &str) -> Result<Vec<Track>, StoreError> {
        let (tracks, _) = self.get_playlist_tracks_paginated(playlist_id, 10000, 0)?;
        Ok(tracks)
    }

    fn get_playlist_tags(&self, playlist_id: &str) -> Result<Vec<String>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT t.name FROM tags t 
             JOIN playlist_tags pt ON t.id = pt.tag_id 
             WHERE pt.playlist_id = ?1",
        )?;
        let tags = stmt
            .query_map(params![playlist_id], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;
        Ok(tags)
    }

    fn get_playlist_track_ids(&self, playlist_id: &str) -> Result<Vec<String>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT track_id FROM playlist_tracks
             WHERE playlist_id = ?1 ORDER BY position ASC",
        )?;
        let ids = stmt
            .query_map(params![playlist_id], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;
        Ok(ids)
    }

    pub fn add_track_to_playlist(
        &self,
        playlist_id: &str,
        track_id: &str,
    ) -> Result<(), StoreError> {
        self.add_tracks_to_playlist_batch(playlist_id, &[track_id.to_string()])
    }

    pub fn add_tracks_to_playlist_batch(
        &self,
        playlist_id: &str,
        track_ids: &[String],
    ) -> Result<(), StoreError> {
        let tx = self.conn.unchecked_transaction()?;
        let now = chrono::Utc::now().timestamp();

        // 获取起始位置
        let mut position: i64 = tx.query_row(
            "SELECT COALESCE(MAX(position), -1) + 1 FROM playlist_tracks WHERE playlist_id = ?1",
            params![playlist_id],
            |row| row.get(0),
        )?;

        for track_id in track_ids {
            tx.execute(
                "INSERT OR IGNORE INTO playlist_tracks (playlist_id, track_id, position, added_at)
                 VALUES (?1, ?2, ?3, ?4)",
                params![playlist_id, track_id, position, now],
            )?;
            position += 1;
        }

        // 更新歌单统计信息
        let count: i64 = tx.query_row(
            "SELECT COUNT(*) FROM playlist_tracks WHERE playlist_id = ?1",
            params![playlist_id],
            |row| row.get(0),
        )?;
        tx.execute(
            "UPDATE playlists SET track_count = ?1, updated_at = ?2 WHERE id = ?3",
            params![count, now, playlist_id],
        )?;

        tx.commit()?;
        Ok(())
    }

    pub fn remove_track_from_playlist(
        &self,
        playlist_id: &str,
        track_id: &str,
    ) -> Result<(), StoreError> {
        self.conn.execute(
            "DELETE FROM playlist_tracks WHERE playlist_id = ?1 AND track_id = ?2",
            params![playlist_id, track_id],
        )?;
        self.update_playlist_track_count(playlist_id)?;
        Ok(())
    }

    pub fn delete_playlist(&self, id: &str) -> Result<(), StoreError> {
        self.conn.execute(
            "DELETE FROM playlist_tracks WHERE playlist_id = ?1",
            params![id],
        )?;
        self.conn
            .execute("DELETE FROM playlists WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn update_playlist(
        &self,
        id: &str,
        name: &str,
        description: &str,
        tags: &[String],
    ) -> Result<(), StoreError> {
        let tx = self.conn.unchecked_transaction()?;
        let now = chrono::Utc::now().timestamp();
        tx.execute(
            "UPDATE playlists SET name = ?1, description = ?2, updated_at = ?3 WHERE id = ?4",
            params![name, description, now, id],
        )?;

        // 更新标签
        tx.execute(
            "DELETE FROM playlist_tags WHERE playlist_id = ?1",
            params![id],
        )?;
        for tag_name in tags {
            tx.execute(
                "INSERT OR IGNORE INTO tags (id, name) VALUES (?1, ?1)",
                params![tag_name],
            )?;
            tx.execute(
                "INSERT OR IGNORE INTO playlist_tags (playlist_id, tag_id) VALUES (?1, ?2)",
                params![id, tag_name],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn update_playlist_cover(
        &self,
        id: &str,
        cover: &Option<String>,
    ) -> Result<(), StoreError> {
        let now = chrono::Utc::now().timestamp();
        self.conn.execute(
            "UPDATE playlists SET cover = ?1, updated_at = ?2 WHERE id = ?3",
            params![cover, now, id],
        )?;
        Ok(())
    }

    fn update_playlist_track_count(&self, playlist_id: &str) -> Result<(), StoreError> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM playlist_tracks WHERE playlist_id = ?1",
            params![playlist_id],
            |row| row.get(0),
        )?;
        let now = chrono::Utc::now().timestamp();
        self.conn.execute(
            "UPDATE playlists SET track_count = ?1, updated_at = ?2 WHERE id = ?3",
            params![count, now, playlist_id],
        )?;
        Ok(())
    }

    pub fn like_album(
        &self,
        album_id: &str,
        title: &str,
        artist: &str,
        cover: Option<&str>,
    ) -> Result<(), StoreError> {
        let now = chrono::Utc::now().timestamp();
        self.conn.execute(
            "INSERT OR REPLACE INTO liked_albums (album_id, album_title, album_artist, album_cover, liked_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![album_id, title, artist, cover, now],
        )?;
        Ok(())
    }

    pub fn unlike_album(&self, album_id: &str) -> Result<(), StoreError> {
        self.conn.execute(
            "DELETE FROM liked_albums WHERE album_id = ?1",
            params![album_id],
        )?;
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
             FROM liked_albums ORDER BY liked_at DESC",
        )?;
        let albums = stmt.query_map([], |row| {
            Ok(LikedAlbum {
                id: row.get(0)?,
                title: row.get(1)?,
                artist: row.get(2)?,
                cover: row.get(3)?,
                liked_at: row.get(4)?,
            })
        })?;
        albums
            .collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::from)
    }

    pub fn get_liked_album_ids(&self) -> Result<Vec<String>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT album_id FROM liked_albums ORDER BY liked_at DESC")?;
        let ids = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;
        Ok(ids)
    }

    pub fn like_artist(&self, artist_name: &str) -> Result<(), StoreError> {
        let now = chrono::Utc::now().timestamp();
        self.conn.execute(
            "INSERT OR IGNORE INTO liked_artists (artist_name, liked_at) VALUES (?1, ?2)",
            params![artist_name, now],
        )?;
        Ok(())
    }

    pub fn unlike_artist(&self, artist_name: &str) -> Result<(), StoreError> {
        self.conn.execute(
            "DELETE FROM liked_artists WHERE artist_name = ?1",
            params![artist_name],
        )?;
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

    pub fn get_liked_artists(&self) -> Result<Vec<Artist>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT a.id, la.artist_name, a.cover_path, a.bio, a.track_count 
             FROM liked_artists la 
             LEFT JOIN artists a ON la.artist_name = a.name 
             ORDER BY la.liked_at DESC",
        )?;
        let artists = stmt
            .query_map([], |row| {
                Ok(Artist {
                    id: row.get::<_, Option<String>>(0)?.unwrap_or_default(),
                    name: row.get(1)?,
                    cover_path: row.get(2)?,
                    bio: row.get(3)?,
                    track_count: row.get::<_, Option<i32>>(4)?.unwrap_or(0),
                })
            })?
            .collect::<Result<Vec<Artist>, _>>()?;
        Ok(artists)
    }

    pub fn add_play_history(&self, history: &PlayHistory) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO play_history (id, track_id, played_at) VALUES (?1, ?2, ?3)",
            params![history.id, history.track_id, history.played_at],
        )?;
        Ok(())
    }

    pub fn get_recently_played(&self, limit: usize) -> Result<Vec<Track>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT t.id, t.path, t.title, t.artist, t.artist_id, t.album, t.album_id, t.duration, t.size, t.bitrate, t.extension, 
                    (SELECT GROUP_CONCAT(g.name, ';') FROM track_genres tg JOIN genres g ON tg.genre_id = g.id WHERE tg.track_id = t.id) as genres,
                    t.added_at, t.mtime, t.cover, t.lyrics, ph.played_at
             FROM tracks t
             INNER JOIN play_history ph ON t.id = ph.track_id
             ORDER BY ph.played_at DESC LIMIT ?1"
        )?;
        let tracks = stmt.query_map(params![limit as i64], |row| {
            let genres_raw: Option<String> = row.get(11)?;
            let genres = genres_raw
                .map(|s| s.split(';').map(|g| g.to_string()).collect())
                .unwrap_or_default();
            Ok(Track {
                id: row.get(0)?,
                path: row.get(1)?,
                title: row.get(2)?,
                artist: row.get(3)?,
                artist_id: row.get(4)?,
                album: row.get(5)?,
                album_id: row.get(6)?,
                duration: row.get(7)?,
                size: row.get(8)?,
                bitrate: row.get(9)?,
                extension: row.get(10)?,
                genres,
                added_at: row.get(12)?,
                mtime: row.get(13)?,
                cover: row.get(14)?,
                lyrics: row.get(15)?,
                played_at: Some(row.get(16)?),
            })
        })?;
        tracks
            .collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::from)
    }

    pub fn clear_play_history(&self) -> Result<(), StoreError> {
        self.conn.execute("DELETE FROM play_history", [])?;
        Ok(())
    }
}
