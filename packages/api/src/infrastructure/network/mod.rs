pub struct NetworkClient {}

impl Default for NetworkClient {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkClient {
    pub fn new() -> Self {
        NetworkClient {}
    }

    pub fn webdav_connect(
        &self,
        _url: &str,
        _username: &str,
        _password: &str,
    ) -> Result<(), String> {
        Ok(())
    }

    pub async fn webdav_upload(&self, _local_path: &str, _remote_path: &str) -> Result<(), String> {
        Ok(())
    }

    pub async fn webdav_download(
        &self,
        _remote_path: &str,
        _local_path: &str,
    ) -> Result<(), String> {
        Ok(())
    }
}
