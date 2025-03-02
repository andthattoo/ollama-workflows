use std::sync::Arc;

use super::types::{Entry, FilePage};
use crate::program::errors::{EmbeddingError, FileSystemError};
use async_trait::async_trait;
use log::debug;
use ollama_rs::generation::embeddings::request::{EmbeddingsInput, GenerateEmbeddingsRequest};
use ollama_rs::Ollama;
use openai_dive::v1::api::Client;
use openai_dive::v1::models::EmbeddingsEngine;
use openai_dive::v1::resources::embedding::{
    EmbeddingEncodingFormat, EmbeddingInput, EmbeddingOutput, EmbeddingParametersBuilder,
};
use serde_json::json;
#[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
use simsimd::SpatialSimilarity;

use text_splitter::TextSplitter;

#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
mod arm_compat {
    pub trait SimpleSpatialSimilarity {
        fn cosine(a: &[f32], b: &[f32]) -> Option<f32>;
    }

    impl SimpleSpatialSimilarity for f32 {
        fn cosine(a: &[f32], b: &[f32]) -> Option<f32> {
            if a.len() != b.len() {
                return None;
            }
            let mut dot_product = 0.0;
            let mut mag_a = 0.0;
            let mut mag_b = 0.0;
            for i in 0..a.len() {
                dot_product += a[i] * b[i];
                mag_a += a[i] * a[i];
                mag_b += b[i] * b[i];
            }
            let magnitude = mag_a.sqrt() * mag_b.sqrt();
            if magnitude == 0.0 {
                None
            } else {
                Some((dot_product / magnitude).clamp(-1.0, 1.0))
            }
        }
    }
}
#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
use self::arm_compat::SimpleSpatialSimilarity;

pub static EMBEDDING_MODEL: &str = "hellord/mxbai-embed-large-v1:f16";

#[async_trait]
pub trait Embedder: Send + Sync {
    async fn generate_embeddings(&self, prompt: &str) -> Result<Vec<f32>, EmbeddingError>;
    async fn generate_query_embeddings(&self, query: &str) -> Result<Vec<f32>, EmbeddingError>;
}

struct OllamaEmbedder {}

impl OllamaEmbedder {
    fn transform_query(query: &str) -> String {
        format!(
            "Represent this sentence for searching relevant passages: {}",
            query
        )
    }
}
#[async_trait]
impl Embedder for OllamaEmbedder {
    async fn generate_embeddings(&self, prompt: &str) -> Result<Vec<f32>, EmbeddingError> {
        let ollama = Ollama::default();
        let res = ollama
            .generate_embeddings(GenerateEmbeddingsRequest::new(
                EMBEDDING_MODEL.to_string(),
                EmbeddingsInput::Single(prompt.to_string()),
            ))
            .await;
        match res {
            Ok(res) => Ok(res.embeddings[0].clone()),
            Err(_) => Err(EmbeddingError::DocumentEmbedding(prompt.to_string())),
        }
    }

    async fn generate_query_embeddings(&self, query: &str) -> Result<Vec<f32>, EmbeddingError> {
        let prompt = OllamaEmbedder::transform_query(query);
        let res = self.generate_embeddings(&prompt).await;
        match res {
            Ok(res) => Ok(res),
            Err(_) => Err(EmbeddingError::QueryEmbedding(query.to_string())),
        }
    }
}

struct OpenAIEmbedder {}

#[async_trait]
impl Embedder for OpenAIEmbedder {
    async fn generate_embeddings(&self, _prompt: &str) -> Result<Vec<f32>, EmbeddingError> {
        let api_key = std::env::var("OPENAI_API_KEY").expect("$OPENAI_API_KEY is not set");
        let client = Client::new(api_key);

        let parameters = EmbeddingParametersBuilder::default()
            .model(EmbeddingsEngine::TextEmbeddingAda002.to_string())
            .input(EmbeddingInput::String(_prompt.to_string()))
            .encoding_format(EmbeddingEncodingFormat::Float)
            .build()
            .map_err(EmbeddingError::BuilderError)?;

        let result = client.embeddings().create(parameters).await;

        match result {
            Ok(result) => {
                let embeddings = result.data[0].clone();
                return match embeddings.embedding {
                    EmbeddingOutput::Float(f64_vec) => {
                        let vec = f64_vec.iter().map(|&x| x as f32).collect();
                        Ok(vec)
                    }
                    _ => Err(EmbeddingError::DocumentEmbedding(
                        "OpenAI embedding result conversion error".to_string(),
                    )),
                };
            }
            Err(_) => Err(EmbeddingError::DocumentEmbedding(
                "OpenAI Embedding response error".to_string(),
            )),
        }
    }

    async fn generate_query_embeddings(&self, _query: &str) -> Result<Vec<f32>, EmbeddingError> {
        self.generate_embeddings(_query).await
    }
}

pub struct FileSystem {
    embedder: Arc<dyn Embedder>,
    entries: Vec<FilePage>,
}

impl FileSystem {
    pub fn new() -> Self {
        if std::env::var("OPENAI_API_KEY").is_ok() {
            FileSystem {
                embedder: Arc::new(OpenAIEmbedder {}),
                entries: Vec::new(),
            }
        } else {
            FileSystem {
                embedder: Arc::new(OllamaEmbedder {}),
                entries: Vec::new(),
            }
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
            if sentence.len() < 25 {
                continue;
            }
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
                    debug!("Similarity: {}, passage: {}", r.1, r.0);
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

    pub async fn have_similar(
        &self,
        query: &Entry,
        threshold: Option<f32>,
    ) -> Result<bool, FileSystemError> {
        let query_embedding = self.embedder.generate_embeddings(&query.to_string()).await;

        let mut thres = 0.85;
        if let Some(threshold) = threshold {
            if !(0.0..=1.0).contains(&threshold) {
                return Err(FileSystemError::InvalidThreshold(threshold));
            }
            thres = threshold;
        }

        match query_embedding {
            Ok(embedding) => {
                let res = self.brute_force_top_n(&embedding, 1);

                let sim = res[0].1;
                if sim > thres {
                    return Ok(true);
                }
                Ok(false)
            }
            Err(err) => Err(FileSystemError::EmbeddingError(err)),
        }
    }

    fn brute_force_top_n(&self, query: &[f32], n: usize) -> Vec<(String, f32)> {
        let mut similarities = Vec::new();
        for (_, v) in &self.entries {
            #[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
            let similarity = f32::cosine(query, v).unwrap_or(0.0) as f32;

            #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
            let similarity = f32::cosine(query, v).unwrap_or(0.0);

            similarities.push(similarity);
        }

        let mut indices: Vec<usize> = (0..similarities.len()).collect();
        indices.sort_by(|&a, &b| similarities[b].partial_cmp(&similarities[a]).unwrap());
        let top_indices: Vec<usize> = indices.into_iter().take(n).collect();

        let top_results: Vec<(String, f32)> = top_indices
            .iter()
            .map(|&i| (self.entries[i].0.clone(), similarities[i]))
            .collect();
        top_results
    }
}
