use std::error::Error;
use std::fmt;

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum CustomError {
    FileSystemError(FileSystemError),
    EmbeddingError(EmbeddingError),
    ToolError(ToolError),
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum FileSystemError {
    InsertionFailed(String),
    SearchError,
    EmbeddingError(EmbeddingError),
    InvalidThreshold(f32),
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum EmbeddingError {
    DocumentEmbedding(String),
    QueryEmbedding(String),
    ModelDoesNotExist,
}

#[derive(Debug)]
pub enum ToolError {
    ToolDoesNotExist,
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CustomError::FileSystemError(err) => write!(f, "File system error: {}", err),
            CustomError::EmbeddingError(err) => write!(f, "Embedding error: {}", err),
            CustomError::ToolError(err) => write!(f, "Tool error: {}", err),
        }
    }
}

impl fmt::Display for FileSystemError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FileSystemError::InsertionFailed(doc) => {
                write!(f, "Insertion failed for document: {}", doc)
            }
            FileSystemError::EmbeddingError(err) => write!(f, "Embedding error: {}", err),
            FileSystemError::SearchError => write!(f, "Search error"),
            FileSystemError::InvalidThreshold(threshold) => write!(f, "Invalid threshold: {}", threshold),
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

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ToolError::ToolDoesNotExist => write!(f, "Tool does not exist"),
        }
    }
}

impl Error for CustomError {}
impl Error for FileSystemError {}
impl Error for EmbeddingError {}
impl Error for ToolError {}

impl From<FileSystemError> for CustomError {
    fn from(err: FileSystemError) -> CustomError {
        CustomError::FileSystemError(err)
    }
}

impl From<EmbeddingError> for CustomError {
    fn from(err: EmbeddingError) -> CustomError {
        CustomError::EmbeddingError(err)
    }
}

impl From<ToolError> for CustomError {
    fn from(err: ToolError) -> CustomError {
        CustomError::ToolError(err)
    }
}
