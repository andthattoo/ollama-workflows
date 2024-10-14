use super::types::{Entry, StackPage, ID};
use std::collections::HashMap;

pub struct Stack {
    pages: HashMap<ID, StackPage>,
}

impl Stack {
    pub fn new() -> Self {
        Stack {
            pages: HashMap::new(),
        }
    }

    pub fn get_all(&self, key: &str) -> Option<&StackPage> {
        self.pages.get(key)
    }

    pub fn peek(&self, key: &str, index: usize) -> Option<Entry> {
        // TODO: add -1 for last element, use isize instead of usize
        let vec = self.pages.get(key);
        if let Some(vec) = vec {
            if index >= vec.len() {
                return None;
            }
            return vec.get(index).cloned();
        }
        None
    }

    pub fn pop(&mut self, key: &str) -> Option<Entry> {
        let page = self.pages.get_mut(key);
        if let Some(page) = page {
            page.pop()
        } else {
            None
        }
    }

    pub fn push(&mut self, key: ID, value: Entry) {
        let page = self.pages.get_mut(&key);
        if let Some(page) = page {
            page.push(value);
        } else {
            self.pages.insert(key, vec![value]);
        }
    }
}
