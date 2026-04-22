#[derive(Clone, PartialEq, Debug)]
pub enum SyncStatus {
    Idle,
    Scanning,
    Uploading,
    Downloading,
    Completed,
    Error(String),
}

#[derive(Clone, PartialEq, Debug)]
pub struct SyncConfig {
    pub webdav_url: String,
    pub username: String,
    pub password: String,
    pub sync_folder: String,
    pub auto_sync: bool,
    pub sync_interval: u32,
}
