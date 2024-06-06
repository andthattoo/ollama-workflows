use std::collections::HashMap;
use super::types::{ID, Entry};

pub struct Cache{
    pages: HashMap<ID, Entry>,
}

impl Cache{
    pub fn new() -> Self{
        Cache{
            pages: HashMap::new(),
        }
    }

    pub fn get(&self, key: &ID) -> Option<&Entry>{
        self.pages.get(key)
    }

    pub fn set(&mut self, key: ID, value: Entry){
        self.pages.insert(key, value);
    }

    pub fn remove(&mut self, key: &str){
        self.pages.remove(key);
    }
}