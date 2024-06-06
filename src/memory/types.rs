pub type ID = String;
pub type StackPage = Vec<Entry>;

//a type that can store both string and json Value
#[derive(Debug, serde::Deserialize)]
pub enum Entry {
    String(String),
    Json(serde_json::Value),
}

impl Entry {
    pub fn to_string(&self) -> String {
        match self {
            Entry::String(s) => s.clone(),
            Entry::Json(j) => j.to_string(),
        }
    }

    //A method that creates an Entry from str by first trying to convert to Value
    pub fn from_str(s: &str) -> Entry {
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

pub enum MemoryReturnType<'a> {
    EntryRef(Option<&'a Entry>), // Replace with the actual type returned by pop
    Entry(Option<Entry>),        // Replace with the actual type returned by search
    EntryVec(Option<Vec<Entry>>), // Replace with the actual type returned by read and peek
}

impl<'a> MemoryReturnType<'a> {
    pub fn is_none(&self) -> bool {
        match self {
            MemoryReturnType::EntryRef(entry_ref) => entry_ref.is_none(),
            MemoryReturnType::Entry(entry) => entry.is_none(),
            MemoryReturnType::EntryVec(entry_vec) => entry_vec.is_none(),
        }
    }
}

impl<'a> MemoryReturnType<'a> {
    pub fn to_string(&self) -> String {
        match self {
            MemoryReturnType::EntryRef(entry_ref) => match entry_ref {
                Some(entry) => entry.to_string(),
                None => String::new(),
            },
            MemoryReturnType::Entry(entry) => match entry {
                Some(entry) => entry.to_string(),
                None => String::new(),
            },
            MemoryReturnType::EntryVec(entry_vec) => {
                let mut result = String::new();
                for entry in entry_vec.iter().flatten() {
                    result.push_str(&entry.to_string());
                }
                result
            }
        }
    }
}
