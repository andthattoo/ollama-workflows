pub mod cache;
pub mod files;
pub mod stack;
pub mod types;

pub use types::MemoryReturnType;

use cache::Cache;
use files::FileSystem;
use stack::Stack;

use std::collections::HashMap;

/// ProgramMemory is a struct that holds the cache, file_system, and stack.
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
    /// Read external memory into Stack
    pub fn read_external_memory(
        &mut self,
        external_memory: &HashMap<types::ID, types::MemoryInputType>,
    ) {
        //match MemoryInputtype enumr
        for (key, value) in external_memory {
            match value {
                types::MemoryInputType::Entry(entry) => {
                    self.write(key.clone(), entry.clone());
                }
                types::MemoryInputType::Page(page) => {
                    for entry in page {
                        self.push(key.clone(), entry.clone());
                    }
                }
            }
        }
    }

    /// Read from the cache.
    pub fn read(&self, key: &types::ID) -> Option<types::Entry> {
        self.cache.get(key)
    }
    /// Write to the cache.
    pub fn write(&mut self, key: types::ID, value: types::Entry) {
        self.cache.set(key, value);
    }
    /// Push to the stack.
    pub fn push(&mut self, key: types::ID, value: types::Entry) {
        self.stack.push(key, value);
    }
    /// Pop from the stack.
    pub fn pop(&mut self, key: &types::ID) -> Option<types::Entry> {
        self.stack.pop(key)
    }
    /// Peek from the stack.
    pub fn peek(&self, key: &str, index: usize) -> Option<types::Entry> {
        self.stack.peek(key, index)
    }
    /// Get all from the stack.
    pub fn get_all(&self, key: &types::ID) -> Option<Vec<types::Entry>> {
        let entries = self.stack.get_all(key);
        entries.map(|entries| entries.to_vec())
    }
    /// Get the size of the stack.
    pub fn size(&self, key: &types::ID) -> usize {
        let entries = self.stack.get_all(key);
        match entries {
            Some(entries) => entries.len(),
            None => 0,
        }
    }
    /// Insert into the file system.
    pub async fn insert(&mut self, doc: &types::Entry) {
        let _ = self.file_system.add(doc).await;
    }
    /// Search the file system.
    pub async fn search(&self, query: &types::Entry) -> Option<Vec<types::Entry>> {
        let resu = self.file_system.search(query).await;
        match resu {
            Ok(res) => Some(res),
            Err(_) => None,
        }
    }

    pub async fn have_similar(&self, query: &str, threshold: Option<f32>) -> Option<bool> {
        let res = self
            .file_system
            .have_similar(&types::Entry::try_value_or_str(query), threshold)
            .await;
        match res {
            Ok(res) => Some(res),
            Err(_) => None,
        }
    }
}
