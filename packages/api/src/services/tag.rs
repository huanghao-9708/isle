use std::collections::HashSet;

pub struct TagService {
    tags: HashSet<String>,
}

impl Default for TagService {
    fn default() -> Self {
        Self::new()
    }
}

impl TagService {
    pub fn new() -> Self {
        TagService {
            tags: HashSet::new(),
        }
    }

    pub fn add_tag(&mut self, tag: &str) {
        self.tags.insert(tag.to_string());
    }

    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.remove(tag);
    }

    pub fn get_all_tags(&self) -> Vec<String> {
        self.tags.iter().cloned().collect()
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(tag)
    }
}
