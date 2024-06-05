use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum CustomError {
    CacheError(CacheError),
    SemanticCacheError(SemanticCacheError),
    StackError(StackError),
    EmbeddingError(EmbeddingError),
    Other(String),
}

#[derive(Debug)]
pub enum CacheError {
    NotFound(String),
    InsertionFailed(String),
}

#[derive(Debug)]
pub enum SemanticCacheError {
    InvalidKey(String),
    InsertionFailed(String),
    SearchError,
    EmbeddingError(EmbeddingError),
}

#[derive(Debug)]
pub enum StackError {
    Overflow,
    Underflow,
}

#[derive(Debug)]
pub enum EmbeddingError {
    DocumentEmbedding(String),
    QueryEmbedding(String),
    ModelDoesNotExist,
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CustomError::CacheError(err) => write!(f, "Cache error: {}", err),
            CustomError::SemanticCacheError(err) => write!(f, "Semantic cache error: {}", err),
            CustomError::StackError(err) => write!(f, "Stack error: {}", err),
            CustomError::Other(msg) => write!(f, "Other error: {}", msg),
            CustomError::EmbeddingError(err) => write!(f, "Embedding error: {}", err),
        }
    }
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CacheError::NotFound(key) => write!(f, "Key not found: {}", key),
            CacheError::InsertionFailed(key) => write!(f, "Failed to insert key: {}", key),
        }
    }
}

impl fmt::Display for SemanticCacheError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SemanticCacheError::InvalidKey(key) => write!(f, "Invalid key: {}", key),
            SemanticCacheError::InsertionFailed(doc) => write!(f, "Insertion failed for document: {}", doc),
            SemanticCacheError::EmbeddingError(err) => write!(f, "Embedding error: {}", err),
            SemanticCacheError::SearchError => write!(f, "Search error"),
        }
    }
}

impl fmt::Display for StackError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StackError::Overflow => write!(f, "Stack overflow"),
            StackError::Underflow => write!(f, "Stack underflow"),
        }
    }
}

impl fmt::Display for EmbeddingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EmbeddingError::DocumentEmbedding(doc) => write!(f, "Error while generating embeddings for doc: {}", doc),
            EmbeddingError::QueryEmbedding(query) => write!(f, "Error while generating embeddings for query: {}", query),
            EmbeddingError::ModelDoesNotExist => write!(f, "Model does not exist. run ollama run hellord/mxbai-embed-large-v1:f16 to create it."),
        }
    }
}

impl Error for CustomError {}
impl Error for CacheError {}
impl Error for SemanticCacheError {}
impl Error for StackError {}
impl Error for EmbeddingError {}

impl From<CacheError> for CustomError {
    fn from(err: CacheError) -> CustomError {
        CustomError::CacheError(err)
    }
}

impl From<SemanticCacheError> for CustomError {
    fn from(err: SemanticCacheError) -> CustomError {
        CustomError::SemanticCacheError(err)
    }
}

impl From<StackError> for CustomError {
    fn from(err: StackError) -> CustomError {
        CustomError::StackError(err)
    }
}

fn main() {
    // Example usage
    let error = CustomError::CacheError(CacheError::NotFound("my_key".to_string()));
    println!("{}", error);
}