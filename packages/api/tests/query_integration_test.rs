use api::models::TrackFilter;
use api::services::LibraryService;
use chrono::Utc;
use std::env;
use std::fs;
use tracing::info;

#[tokio::test]
async fn test_library_query_and_filter() {
    let mut temp_dir = env::temp_dir();
    temp_dir.push(format!("isle_query_test_{}", Utc::now().timestamp()));
    fs::create_dir_all(&temp_dir).unwrap();

    let db_path = temp_dir.join("query_test.db");
    let music_dir = temp_dir.join("music_samples");
    fs::create_dir_all(&music_dir).expect("Failed to create music samples dir");

    // 1. 准备多首模拟歌曲
    let songs = [
        ("Jay Chou", "Sunny Day", "Ye Hui Mei", "Pop"),
        ("Jay Chou", "Blue and White Porcelain", "On the Run", "Pop"),
        ("Eason Chan", "Ten Years", "U87", "Pop"),
        ("Eason Chan", "King of Songs", "Special", "Rock"),
        ("Taylor Swift", "Shake It Off", "1989", "Country"),
    ];

    for (artist, title, _album, _genre) in &songs {
        let filename = format!("{} - {}.mp3", artist, title);
        let file_path = music_dir.join(&filename);
        fs::write(
            &file_path,
            format!("mock audio binary data for {} - {}", artist, title),
        )
        .expect("Failed to write mock song");
    }

    let mut library = LibraryService::new();
    library.init_database(db_path).unwrap();

    // 使用规范化后的绝对路径
    let music_dir_abs = fs::canonicalize(&music_dir).expect("Failed to canonicalize music dir");
    let music_dir_str = music_dir_abs.to_str().expect("Path to string failed");

    // 由于 Windows 上的 canonicalize 会添加 \\?\ 前缀，我们需要处理一下
    let music_dir_str = music_dir_str.trim_start_matches(r"\\?\");

    // 扫描文件夹来填充数据
    library
        .add_scan_folder(music_dir_str, true)
        .await
        .expect("Add scan folder failed");

    // 2. 检查基本计数
    let stats = library.stats();
    assert!(
        stats.track_count >= 5,
        "应该扫描到 5 个音轨，实际为 {}",
        stats.track_count
    );

    // 3. 测试关键字搜索（模糊匹配）
    info!("Testing keyword search...");
    let result = library.search_tracks("Jay", 1, 10).unwrap();
    assert_eq!(result.total, 2, "Keyword 'Jay' should find 2 tracks");

    let result_eason = library.search_tracks("Ten", 1, 10).unwrap();
    assert_eq!(result_eason.total, 1);
    assert!(result_eason.items[0].title.contains("Ten"));

    // 4. 测试分页
    let result_page = library.search_tracks("a", 1, 2).unwrap();
    assert_eq!(result_page.items.len(), 2);
    assert!(result_page.total >= 2);

    // 5. 测试多维度过滤
    // 注意：当前实现中 album 默认为 "Unknown"，genre 默认为 None
    tracing::info!("Testing multi-filter with defaults...");
    let filter = TrackFilter {
        artist: Some("Jay Chou".to_string()),
        album: Some("Unknown".to_string()), // 目前 scan_folder 的限制
        ..Default::default()
    };
    let filter_result = library.filter_tracks(filter, 1, 10).unwrap();
    assert_eq!(
        filter_result.total, 2,
        "Jay Chou should have 2 unknown album tracks"
    );

    // 6. 资源清理
    let _ = fs::remove_dir_all(&temp_dir);
}
