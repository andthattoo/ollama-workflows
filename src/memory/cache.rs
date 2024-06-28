use super::types::{Entry, ID};
use std::collections::HashMap;

pub struct Cache {
    pages: HashMap<ID, Entry>,
}

impl Cache {
    pub fn new() -> Self {
        Cache {
            pages: HashMap::new(),
        }
    }

    pub fn get(&self, key: &ID) -> Option<Entry> {
        self.pages.get(key).cloned()
    }

    pub fn set(&mut self, key: ID, value: Entry) {
        self.pages.insert(key, value);
    }
}
