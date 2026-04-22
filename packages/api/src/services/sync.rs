use crate::infrastructure::NetworkClient;
use crate::models::{SyncConfig, SyncStatus};

pub struct SyncService {
    client: NetworkClient,
    status: SyncStatus,
    config: Option<SyncConfig>,
}

impl Default for SyncService {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncService {
    pub fn new() -> Self {
        SyncService {
            client: NetworkClient::new(),
            status: SyncStatus::Idle,
            config: None,
        }
    }

    pub fn set_config(&mut self, config: SyncConfig) {
        self.config = Some(config);
    }

    pub async fn sync(&mut self) -> Result<(), String> {
        let config = self.config.as_ref().ok_or("No sync config")?;

        self.status = SyncStatus::Scanning;

        self.client
            .webdav_connect(&config.webdav_url, &config.username, &config.password)?;

        self.status = SyncStatus::Uploading;

        self.status = SyncStatus::Downloading;

        self.status = SyncStatus::Completed;

        Ok(())
    }

    pub fn status(&self) -> &SyncStatus {
        &self.status
    }

    pub fn cancel(&mut self) {
        self.status = SyncStatus::Idle;
    }
}
