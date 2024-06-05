use std::collections::HashMap;
use super::types::{WorkflowID, CacheEntry};

//implement a simple cache

pub struct Cache{
    pages: HashMap<WorkflowID, CacheEntry>,
}

impl Cache{
    pub fn new() -> Self{
        Cache{
            pages: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&CacheEntry>{
        self.pages.get(key)
    }

    pub fn set(&mut self, key: String, value: CacheEntry){
        self.pages.insert(key, value);
    }

    pub fn remove(&mut self, key: &str){
        self.pages.remove(key);
    }
}