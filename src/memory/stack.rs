use std::collections::HashMap; 
use super::types::{WorkflowID, StackEntry, StackPage};

pub struct Stack {
    pages: HashMap<WorkflowID, StackPage>
}

impl Stack {
    pub fn new() -> Self {
        Stack {
            pages: HashMap::new()
        }
    }

    pub fn peek(&self, key: &str, index: u32) -> Option<&StackEntry> {
        let vec = self.pages.get(key);
        if let Some(vec) = vec {
            return vec.get(index as usize);
        }
        else{
            return None;
        }
    }

    pub fn pop(&mut self, key: &str)-> Option<StackEntry> {
        let page = self.pages.get_mut(key);
        if let Some(page) = page {
            page.pop()
        }
        else {
            None
        }
    }

    pub fn push(&mut self, key: WorkflowID, value: StackEntry) {
        let page = self.pages.get_mut(&key);
        if let Some(page) = page {
            page.push(value);
        }
        else{
            let mut new_page = StackPage::new();
            new_page.push(value);
            self.pages.insert(key, new_page);
        }
    }

    pub fn remove(&mut self, key: &str) {
        self.pages.remove(key);
    }
}
