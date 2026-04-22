pub struct CryptoEngine {}

impl Default for CryptoEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl CryptoEngine {
    pub fn new() -> Self {
        CryptoEngine {}
    }

    pub fn encrypt(&self, data: &[u8], _key: &[u8]) -> Result<Vec<u8>, String> {
        Ok(data.to_vec())
    }

    pub fn decrypt(&self, data: &[u8], _key: &[u8]) -> Result<Vec<u8>, String> {
        Ok(data.to_vec())
    }

    pub fn hash(&self, _data: &[u8]) -> Vec<u8> {
        vec![]
    }

    pub fn generate_key(&self) -> Vec<u8> {
        vec![]
    }
}
