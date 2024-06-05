use ollama_rs::Ollama;
use usearch::{Index, IndexOptions, MetricKind, ScalarKind, new_index};
use crate::core::errors::{SemanticCacheError, EmbeddingError};
use super::types::SemanticCacheEntry;

pub static EMBEDDING_MODEL: &str = "hellord/mxbai-embed-large-v1:f16";
struct Embedder{
    ollama: Ollama,
    model: String,
}

impl Embedder{
    pub fn new() -> Self{
        Embedder{
            ollama: Ollama::default(),
            model: EMBEDDING_MODEL.to_string(),
        }
    }

    pub async fn generate_embeddings(&self, prompt: &str) -> Result<Vec<f64>, EmbeddingError>{
        let res = self.ollama
            .generate_embeddings(self.model.clone(), prompt.to_string(), None)
            .await;

        match res{
            Ok(res) => Ok(res.embeddings),
            Err(_) => Err(EmbeddingError::DocumentEmbedding(prompt.to_string()))
        }
    }

    pub async fn generate_query_embeddings(&self, query: &str) -> Result<Vec<f64>, EmbeddingError>{
        let prompt = Embedder::transform_query(query);
        let res = self.generate_embeddings(&prompt).await;
        match res{
            Ok(res) => Ok(res),
            Err(_) => Err(EmbeddingError::QueryEmbedding(query.to_string()))
            
        }
    }

    fn transform_query(query: &str) -> String {
        format!("Represent this sentence for searching relevant passages: {}", query)
    }
}


pub struct SemanticCache{
    embedder: Embedder,
    index: Index,
    documents: Vec<String>,
}

impl SemanticCache{
    pub fn new() -> Self{

        let options = IndexOptions {
            dimensions: 3, // necessary for most metric kinds
            metric: MetricKind::IP, // or MetricKind::L2sq, MetricKind::Cos ...
            quantization: ScalarKind::F16, // or ScalarKind::F32, ScalarKind::I8, ScalarKind::B1x8 ...
            connectivity: 0, // zero for auto
            expansion_add: 0, // zero for auto
            expansion_search: 0, // zero for auto
            multi: true,
        };
       
        SemanticCache{
            embedder: Embedder::new(),
            index:  new_index(&options).unwrap(),
            documents: Vec::new(),
        }
    }

    pub fn load_model(&self){
        self.index.load("index.usearch").unwrap();
    }
    
    pub fn save_model(&self){
        self.index.save("index.usearch").unwrap();
    }

    pub async fn add(&mut self, doc: &SemanticCacheEntry)->Result<(), SemanticCacheError>{
        let embedding = self.embedder.generate_embeddings(doc).await;
        match embedding{
            Ok(embedding) => {
                let res = self.index.add( self.index.size() as u64, &embedding);
                match res{
                    Ok(_) => {
                        self.documents.push(doc.to_string());
                        Ok(())
                    },
                    Err(_) => Err(SemanticCacheError::InsertionFailed(doc.to_string()))
                }
            },
            Err(err) => Err(SemanticCacheError::EmbeddingError(err))
        }
    }

    pub async fn search(&self, query: &SemanticCacheEntry) -> Result<Vec<SemanticCacheEntry>, SemanticCacheError>
    {
        let embedding = self.embedder.generate_query_embeddings(query).await;
        match embedding{
            Ok(embedding) => {
                let results = self.index.search(&embedding, 10);
                match results{
                    Ok(res) => {
                        let mut passages = Vec::new();
                        for (key, distance) in res.keys.iter().zip(res.distances.iter()) {
                            if distance > &0.5 {
                                break;
                            };
                            let doc = &self.documents[*key as usize];
                            passages.push(doc.to_string());
                        }
                        Ok(passages)
                    },
                    Err(_) => Err(SemanticCacheError::SearchError)
                }
            },
            Err(err) => Err(SemanticCacheError::EmbeddingError(err))
        }
    }
}