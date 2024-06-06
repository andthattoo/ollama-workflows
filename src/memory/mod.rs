pub mod cache;
pub mod files;
pub mod stack;
pub mod types;

pub use types::MemoryReturnType;

use cache::Cache;
use files::FileSystem;
use stack::Stack;


pub struct ProgramMemory {
    cache: Cache,
    file_system: FileSystem,
    stack: Stack,
}

impl ProgramMemory {
    pub fn new() -> Self {
        ProgramMemory {
            cache: Cache::new(),
            file_system: FileSystem::new(),
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

    pub fn read(&self, key: &types::ID) -> Option<&types::Entry> {
        self.cache.get(key)
    }

    pub fn write(&mut self, key: types::ID, value: types::Entry) {
        self.cache.set(key, value);
    }

    pub fn push(&mut self, key: types::ID, value: types::Entry) {
        self.stack.push(key, value);
    }

    pub fn pop(&mut self, key: &types::ID) -> Option<types::Entry> {
        self.stack.pop(key)
    }

    pub fn peek(&self, key: &str, index: usize) -> Option<&types::Entry> {
        self.stack.peek(key, index)
    }

    pub async fn insert(&mut self, doc: &types::Entry) {
        self.file_system.add(doc).await;
    }

    pub async fn search(&self, query: &types::Entry) -> Option<Vec<types::Entry>> {
        let resu = self.file_system.search(query).await;
        match resu {
            Ok(res) => Some(res),
            Err(_) => None,
        }
    }
    
}