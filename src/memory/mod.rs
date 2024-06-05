pub mod cache;
pub mod semantic;
pub mod stack;
pub mod types;

use cache::Cache;
use semantic::SemanticCache;
use stack::Stack;

pub struct ProgramMemory {
    cache: Cache,
    semantic_cache: SemanticCache,
    stack: Stack,
}

impl ProgramMemory {
    pub fn new() -> Self {
        ProgramMemory {
            cache: Cache::new(),
            semantic_cache: SemanticCache::new(),
            stack: Stack::new(),
        }
    }
}

impl Default for ProgramMemory {
    fn default() -> Self {
        ProgramMemory::new()
    }
}

impl ProgramMemory {

    pub fn read(&self, key: &str) -> Option<&types::CacheEntry> {
        self.cache.get(key)
    }

    pub fn write(&mut self, key: String, value: types::CacheEntry) {
        self.cache.set(key, value);
    }

    pub fn push(&mut self, key: types::WorkflowID, value: types::StackEntry) {
        self.stack.push(key, value);
    }

    pub fn pop(&mut self, key: &str) {
        self.stack.pop(key);
    }

    pub fn peek(&self, key: &str, index: u32) -> Option<&types::StackEntry> {
        self.stack.peek(key, index)
    }

    pub async fn insert(&mut self, doc: types::SemanticCacheEntry) {
        self.semantic_cache.add(&doc).await;
    }

    pub async fn search(&self, query: &types::SemanticCacheEntry) -> Option<Vec<types::SemanticCacheEntry>> {
        let resu = self.semantic_cache.search(query).await;
        match resu {
            Ok(res) => Some(res),
            Err(_) => None,
        }
    }
    
}