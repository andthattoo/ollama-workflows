use super::types::Entry;
use crate::program::errors::{EmbeddingError, FileSystemError};
use ollama_rs::Ollama;
use text_splitter::TextSplitter;
use usearch::{new_index, Index, IndexOptions, MetricKind, ScalarKind};

pub static EMBEDDING_MODEL: &str = "hellord/mxbai-embed-large-v1:f16";
struct Embedder {
    ollama: Ollama,
    model: String,
}

impl Embedder {
    pub fn new() -> Self {
        Embedder {
            ollama: Ollama::default(),
            model: EMBEDDING_MODEL.to_string(),
        }
    }

    pub async fn generate_embeddings(&self, prompt: &str) -> Result<Vec<f64>, EmbeddingError> {
        let res = self
            .ollama
            .generate_embeddings(self.model.clone(), prompt.to_string(), None)
            .await;

        match res {
            Ok(res) => Ok(res.embeddings),
            Err(_) => Err(EmbeddingError::DocumentEmbedding(prompt.to_string())),
        }
    }

    pub async fn generate_query_embeddings(&self, query: &str) -> Result<Vec<f64>, EmbeddingError> {
        let prompt = Embedder::transform_query(query);
        let res = self.generate_embeddings(&prompt).await;
        match res {
            Ok(res) => Ok(res),
            Err(_) => Err(EmbeddingError::QueryEmbedding(query.to_string())),
        }
    }

    fn transform_query(query: &str) -> String {
        format!(
            "Represent this sentence for searching relevant passages: {}",
            query
        )
    }
}

pub struct FileSystem {
    embedder: Embedder,
    index: Index,
    documents: Vec<String>,
}

impl FileSystem {
    pub fn new() -> Self {
        let options = IndexOptions {
            dimensions: 3,                 // necessary for most metric kinds
            metric: MetricKind::IP,        // or MetricKind::L2sq, MetricKind::Cos ...
            quantization: ScalarKind::F16, // or ScalarKind::F32, ScalarKind::I8, ScalarKind::B1x8 ...
            connectivity: 0,               // zero for auto
            expansion_add: 0,              // zero for auto
            expansion_search: 0,           // zero for auto
            multi: true,
        };

        FileSystem {
            embedder: Embedder::new(),
            index: new_index(&options).unwrap(),
            documents: Vec::new(),
        }
    }

    pub async fn add(&mut self, entry: &Entry) -> Result<(), FileSystemError> {
        let doc = match entry {
            Entry::String(s) => s,
            Entry::Json(j) => j.as_str().unwrap(),
        };

        let splitter = TextSplitter::new(1000);
        let chunks = splitter.chunks(doc);
        let sentences: Vec<String> = chunks.map(|s| s.to_string()).collect();

        for sentence in sentences {
            let embedding = self.embedder.generate_embeddings(&sentence).await;
            let res = match embedding {
                Ok(embedding) => {
                    let res = self.index.add(self.index.size() as u64, &embedding);
                    match res {
                        Ok(_) => {
                            self.documents.push(doc.to_string());
                            Ok(())
                        }
                        Err(_) => Err(FileSystemError::InsertionFailed(doc.to_string())),
                    }
                }
                Err(err) => Err(FileSystemError::EmbeddingError(err)),
            };
            res?;
        }

        Ok(())
    }

    pub async fn search(&self, query: &Entry) -> Result<Vec<Entry>, FileSystemError> {
        let embedding = self
            .embedder
            .generate_query_embeddings(&query.to_string())
            .await;
        match embedding {
            Ok(embedding) => {
                let results = self.index.search(&embedding, 10);
                match results {
                    Ok(res) => {
                        let mut passages = Vec::new();
                        for (key, distance) in res.keys.iter().zip(res.distances.iter()) {
                            if distance > &0.5 {
                                break;
                            };
                            let doc = &self.documents[*key as usize];
                            let passage = Entry::try_value_or_str(doc);
                            passages.push(passage);
                        }
                        Ok(passages)
                    }
                    Err(_) => Err(FileSystemError::SearchError),
                }
            }
            Err(err) => Err(FileSystemError::EmbeddingError(err)),
        }
    }
}
