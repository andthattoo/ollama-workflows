use serde::{Deserialize, Serialize};
use std::fmt;
pub type ID = String;
pub type StackPage = Vec<Entry>;
pub type FilePage = (String, Vec<f32>);

/// Entry is an enum that can be either a String or a Json Value.
/// It is used for I/O operations in the memory module.
#[derive(Debug, Serialize, serde::Deserialize, PartialEq)]
pub enum Entry {
    String(String),
    Json(serde_json::Value),
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Entry::String(s) => write!(f, "{}", s),
            Entry::Json(j) => write!(f, "{}", j),
        }
    }
}

impl Entry {
    //A method that creates an Entry from str by first trying to convert to Value
    pub fn try_value_or_str(s: &str) -> Entry {
        match serde_json::from_str(s) {
            Ok(json) => Entry::Json(json),
            Err(_) => Entry::String(s.to_string()),
        }
    }
}
impl Clone for Entry {
    fn clone(&self) -> Self {
        match self {
            Entry::String(s) => Entry::String(s.clone()),
            Entry::Json(j) => Entry::Json(j.clone()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryInputType {
    Entry(Entry),
    Page(StackPage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryReturnType {
    //<'a>
    //EntryRef(Option<&'a Entry>),
    Entry(Option<Entry>),
    EntryVec(Option<Vec<Entry>>),
    Multiple(Vec<MemoryReturnType>),
}

impl MemoryReturnType {
    pub fn is_none(&self) -> bool {
        match self {
            //MemoryReturnType::EntryRef(entry_ref) => entry_ref.is_none(),
            MemoryReturnType::Entry(entry) => entry.is_none(),
            MemoryReturnType::EntryVec(entry_vec) => entry_vec.is_none(),
            MemoryReturnType::Multiple(returns) => returns.is_empty(),
        }
    }

    pub fn to_json(&self) -> Option<String> {
        match self {
            MemoryReturnType::EntryVec(Some(entries)) => {
                // Extracting the string directly from each Entry.
                let values: Vec<String> = entries
                    .iter()
                    .map(|entry| entry.to_string().clone())
                    .collect();
                let res = serde_json::to_string(&values);
                match res {
                    Ok(json) => Some(json),
                    Err(_) => None,
                }
            }
            MemoryReturnType::Multiple(returns) => {
                let values: Vec<String> = returns.iter().map(|ret| ret.to_string()).collect();
                serde_json::to_string(&values).ok()
            }
            _ => None,
        }
    }
}

impl fmt::Display for MemoryReturnType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            /*MemoryReturnType::EntryRef(entry_ref) => match entry_ref {
                Some(entry) => write!(f, "{}", entry),
                None => write!(f, ""),
            },*/
            MemoryReturnType::Entry(entry) => match entry {
                Some(entry) => write!(f, "{}", entry),
                None => write!(f, ""),
            },
            MemoryReturnType::EntryVec(entry_vec) => {
                let mut result = String::new();
                for entry in entry_vec.iter().flatten() {
                    result.push_str(&entry.to_string());
                    result.push_str(" \n"); // Add a newline in between strings
                }
                write!(f, "{}", result)
            }
            MemoryReturnType::Multiple(returns) => {
                let mut result = String::new();
                for ret in returns {
                    result.push_str(&ret.to_string());
                    result.push_str(" \n");
                }
                write!(f, "{}", result)
            }
        }
    }
}

impl From<MemoryReturnType> for Vec<Entry> {
    fn from(memory_return: MemoryReturnType) -> Vec<Entry> {
        match memory_return {
            MemoryReturnType::EntryVec(entry_vec) => entry_vec.unwrap_or_default(),
            _ => vec![],
        }
    }
}
