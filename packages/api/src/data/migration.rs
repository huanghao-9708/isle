pub struct Migrator {}

impl Default for Migrator {
    fn default() -> Self {
        Self::new()
    }
}

impl Migrator {
    pub fn new() -> Self {
        Migrator {}
    }

    pub fn migrate(&self) -> Result<(), String> {
        Ok(())
    }

    pub fn get_current_version(&self) -> u32 {
        1
    }
}
