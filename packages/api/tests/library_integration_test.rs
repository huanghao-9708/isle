use api::services::LibraryService;
use std::fs;
use tracing::info;

#[tokio::test]
async fn test_library_scan_and_cleanup() {
    // 1. 准备测试环境：创建临时数据库和音乐目录
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test_library.db");
    let music_dir = temp_dir.path().join("Music");
    fs::create_dir(&music_dir).expect("Failed to create music dir");

    // 创建一些模拟音乐文件
    let f1 = music_dir.join("Artist 1 - Track 1.mp3");
    let f2 = music_dir.join("Artist 2 - Track 2.mp3");
    fs::write(&f1, "fake mp3 data 1").expect("Failed to write f1");
    fs::write(&f2, "fake mp3 data 2").expect("Failed to write f2");

    // 2. 初始化服务
    let mut library = LibraryService::new();
    library
        .init_database(db_path)
        .expect("Failed to init database");

    let music_dir_str = music_dir
        .to_str()
        .expect("Path conversion failed")
        .replace('\\', "/");

    // 3. 执行添加与扫描测试
    info!("Testing add_scan_folder: {}", music_dir_str);
    library
        .add_scan_folder(&music_dir_str, true)
        .await
        .expect("Failed to add and scan folder");

    let tracks = library.get_all_tracks().expect("Failed to get tracks");
    assert_eq!(
        tracks.len(),
        2,
        "Library should contain 2 tracks after scan"
    );

    let folders = library.get_scan_folders().expect("Failed to get folders");
    assert!(
        folders.iter().any(|f| f.path == music_dir_str),
        "Folder list should contain music dir"
    );

    // 4. 执行移除与清理测试
    tracing::info!("Testing remove_scan_folder");
    library
        .remove_scan_folder(&music_dir_str)
        .expect("Failed to remove scan folder");

    let stats_after = library.stats();
    assert_eq!(
        stats_after.track_count, 0,
        "Tracks should be cleared after folder removal"
    );

    let tracks_after = library.get_all_tracks().unwrap();
    assert!(tracks_after.is_empty(), "Database should be empty");

    // 5. 资源清理
    let _ = fs::remove_dir_all(&temp_dir);
}
