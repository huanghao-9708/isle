# 「私屿」音乐库功能实施方案 (Music Library Implementation)

**文档版本**：V1.0
**修订日期**：2026年04月
**状态**：核心功能已完成 (关联逻辑与数据同步已实现)
**关联文档**：@project.md (需求锚定), @technical_solution.md (技术方案)

---

## 一、 功能概述

音乐库模块是「私屿」的核心基础设施，负责本地音频文件的深度扫描、元数据解析、分类索引及多维度统计。本方案旨在提供类似网易云音乐“本地音乐”的流畅体验，同时严守“数据绝对私有”的底线。

---

## 二、 核心功能需求

### 2.1 扫描管理 (File Scanning)

支持用户自主控制扫描范围，确保扫描过程轻量且不干扰播放。

1. **扫描路径管理**：
    * **添加文件夹**：用户可手动添加多个本地目录作为扫描源。
    * **移除文件夹**：支持移除路径（仅从库中删除索引，不删除物理文件）。
    * **黑名单设置**：支持排除特定文件夹（如系统音频目录、隐藏文件夹）。
2. **扫描策略**：
    * **全量扫描**：首次添加路径或用户手动触发。
    * **增量扫描**：利用文件修改时间（mtime）和大小（size）判断，仅解析变更文件。
    * **实时监控**：(P1) 基于 `notify` 库监听文件系统变动，实现即时更新。
3. **过滤器设置**：
    * **按时长过滤**：忽略少于 60 秒的音频（排除系统提示音、广告片段）。
    * **按大小过滤**：忽略小于 1MB 的音频。

### 2.2 统计与分类 (Statistics & Categorization)

参照网易云音乐，提供四级导航：**单曲、歌手、专辑、文件夹**。现在已实现完整的实体关联。

1. **数据统计看板**：
    * 实时显示：歌曲总数、专辑总数、艺术家总数、流派总数、文件夹总数。
2. **分类视图**：
    * **单曲 (Songs)**：按添加时间、歌名、首字母排序。
    * **歌手 (Artists)**：提取 `Artist` 标签，生成唯一 ID，支持封面与简介。
    * **专辑 (Albums)**：提取 `Album` 标签，归属于特定歌手，支持专辑简介。
    * **流派 (Genres)**：按 `Genre` 标签分类。
    * **文件夹 (Folders)**：保持物理目录结构视图，方便按路径找歌。

### 2.3 分类查询与搜索

1. **多级筛选**：在歌手/专辑视图下，支持二次过滤（例如：在“周杰伦”下搜索“晴天”）。
2. **实时搜索**：全局本地搜索，支持拼音首字母匹配（如搜索 "ZJL" 匹配 "周杰伦"）。

---

## 三、 技术实现设计

### 3.1 数据库架构 (SQLite Schema)

为保证 TB 级音乐库的毫秒级查询，设计并实现了以下关系型表结构：

```sql
-- 歌曲核心表
CREATE TABLE tracks (
    id TEXT PRIMARY KEY,           -- 文件哈希 (SHA-256)
    path TEXT UNIQUE,              -- 物理路径
    title TEXT,                    -- 歌名
    artist TEXT,                   -- 艺术家名称 (冗余用于展示)
    artist_id TEXT,                -- 关联艺术家 ID (哈希生成的唯一标识)
    album TEXT,                    -- 专辑名称 (冗余用于展示)
    album_id TEXT,                 -- 关联专辑 ID (哈希生成的唯一标识)
    duration INTEGER,              -- 时长(秒)
    size INTEGER,                  -- 文件大小
    bitrate INTEGER,               -- 比特率
    extension TEXT,                -- 后缀名(flac, mp3等)
    genre TEXT,                    -- 流派 (冗余字段或默认流派)
    added_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    mtime INTEGER,                 -- 文件最后修改时间
    cover TEXT,                    -- 封面缓存路径
    FOREIGN KEY (artist_id) REFERENCES artists(id),
    FOREIGN KEY (album_id) REFERENCES albums(id)
);

-- 艺术家表
CREATE TABLE artists (
    id TEXT PRIMARY KEY,           -- SHA-256(ArtistName)
    name TEXT UNIQUE,
    cover_path TEXT,               -- 艺术家封面/头像缓存路径
    bio TEXT,                      -- 艺术家简介
    track_count INTEGER DEFAULT 0
);

-- 专辑表
CREATE TABLE albums (
    id TEXT PRIMARY KEY,           -- SHA-256(ArtistName + AlbumTitle)
    title TEXT,
    artist_id TEXT,                -- 关联艺术家表 ID
    cover_path TEXT,               -- 提取的封面缓存路径
    description TEXT,              -- 专辑简介
    track_count INTEGER DEFAULT 0,
    FOREIGN KEY (artist_id) REFERENCES artists(id)
);

-- 流派表
CREATE TABLE genres (
    id INTEGER PRIMARY KEY AUTOINCREMENT, -- 自增内部编码
    name TEXT UNIQUE,              -- 流派名称 (如 Rock, Pop)
    image TEXT,                    -- 流派封面图路径
    description TEXT               -- 流派介绍
);

-- 曲目流派关联表 (多对多)
CREATE TABLE track_genres (
    track_id TEXT,
    genre_id INTEGER,              -- 关联流派 ID
    PRIMARY KEY (track_id, genre_id),
    FOREIGN KEY (track_id) REFERENCES tracks(id) ON DELETE CASCADE,
    FOREIGN KEY (genre_id) REFERENCES genres(id) ON DELETE CASCADE
);
```

### 3.2 扫描流程 (Rust Implementation)

基于 `tokio` 异步运行时与 `Symphonia` 解码库。

1. **多线程遍历**：使用 `walkdir` 或并行 `jwalk` 快速获取音频文件列表。
2. **元数据提取**：
    * 调用 `Symphonia` 探测文件头。
    * 提取：Title, Artist, Album, Year, Genre, Duration。
    * **封面提取**：提取嵌入的图片并压缩缓存至本地应用数据目录（防止直接读取大图卡顿）。
3. **批处理写入**：利用 SQLite 的 `BEGIN TRANSACTION` 进行批量插入，优化 IO。

### 3.3 UI 组件拆分 (Dioxus)

* `LibraryHeader`：展示统计数量（歌曲、歌手等）。
* `ScannerModal`：扫描进度条、路径配置弹窗。
* `TrackTable`：虚拟列表组件，承载歌曲清单。
* `CategoryGrid`：封面墙或列表形式展示歌手/专辑。

---

## 四、 隐私与性能准则

1. **100% 本地化**：所有统计、解析、封面匹配逻辑均在用户本机运行。绝不访问远程数据库获取元数据。
2. **零阻塞 UI**：扫描任务在低优先级后台线程运行。UI 通过 `Signal` 异步接收进度更新。
3. **极致索引**：SQLite 必须对 `title`, `artist`, `path` 建立索引，确保万级数据下搜索无延迟。

---

## 五、 后续迭代规划 (P1)

1. **拼音支持**：集成 `pinyin` 库，支持中文歌手名的拼音排序。
2. **智能归类**：针对没有标签的歌曲，尝试通过文件名猜测 `歌手 - 歌名`。
3. **文件夹深度统计**：显示每个物理目录的占用空间与音频质量分布。
