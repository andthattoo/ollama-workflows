pub type WorkflowID = String;
pub type StackEntry = String;
pub type StackPage = Vec<StackEntry>;
pub type SemanticCacheEntry = String;

//a type that can store both string and json Value
#[derive(Debug, serde::Deserialize)]
pub enum CacheEntry {
    String(String),
    Json(serde_json::Value),
}