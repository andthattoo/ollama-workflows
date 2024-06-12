use super::types::{Entry, FilePage};
use crate::program::errors::{EmbeddingError, FileSystemError};
use ollama_rs::Ollama;
use serde_json::json;
use simsimd::SpatialSimilarity;
use text_splitter::TextSplitter;

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

    pub async fn generate_embeddings(&self, prompt: &str) -> Result<Vec<f32>, EmbeddingError> {
        let res = self
            .ollama
            .generate_embeddings(self.model.clone(), prompt.to_string(), None)
            .await;
        match res {
            Ok(res) => Ok(res.embeddings.iter().map(|&x| x as f32).collect()),
            Err(_) => Err(EmbeddingError::DocumentEmbedding(prompt.to_string())),
        }
    }

    pub async fn generate_query_embeddings(&self, query: &str) -> Result<Vec<f32>, EmbeddingError> {
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
    entries: Vec<FilePage>,
}

impl FileSystem {
    pub fn new() -> Self {
        FileSystem {
            embedder: Embedder::new(),
            entries: Vec::new(),
        }
    }

    pub async fn add(&mut self, entry: &Entry) -> Result<(), FileSystemError> {
        let doc = match entry {
            Entry::String(s) => s,
            Entry::Json(j) => j.as_str().unwrap(),
        };

        let splitter = TextSplitter::new(250);
        let chunks = splitter.chunks(doc);
        let sentences: Vec<String> = chunks.map(|s| s.to_string()).collect();

        for sentence in sentences {
            let embedding = self.embedder.generate_embeddings(&sentence).await;
            match embedding {
                Ok(embedding) => {
                    //convert to f32
                    self.entries.push((sentence.to_string(), embedding));
                }
                Err(err) => return Err(FileSystemError::EmbeddingError(err)),
            }
        }

        Ok(())
    }

    pub async fn search(&self, query: &Entry) -> Result<Vec<Entry>, FileSystemError> {
        let query_embedding = self
            .embedder
            .generate_query_embeddings(&query.to_string())
            .await;
        match query_embedding {
            Ok(embedding) => {
                //to f32
                let res = self.brute_force_top_n(&embedding, 3);

                let mut passages = Vec::new();
                for r in res {
                    //can add distance threshold here
                    let entry = Entry::Json(json!({
                        "passage": r.0,
                        "similarity": r.1
                    }));
                    passages.push(entry);
                }
                Ok(passages)
            }
            Err(err) => Err(FileSystemError::EmbeddingError(err)),
        }
    }

    fn brute_force_top_n(&self, query: &[f32], n: usize) -> Vec<(String, f32)> {
        let mut distances = Vec::new();
        for (_, v) in &self.entries {
            let distance = f32::cosine(query, v).unwrap_or(0.0) as f32;
            distances.push(distance);
        }

        let mut indices: Vec<usize> = (0..distances.len()).collect();
        indices.sort_by(|&a, &b| distances[a].partial_cmp(&distances[b]).unwrap());
        let top_indices: Vec<usize> = indices.into_iter().take(n).collect();
        //Collect into (String, f32)
        let top_results: Vec<(String, f32)> = top_indices
            .iter()
            .map(|&i| (self.entries[i].0.clone(), distances[i]))
            .collect();
        top_results
    }
}
