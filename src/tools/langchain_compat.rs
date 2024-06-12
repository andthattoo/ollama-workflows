use std::{error::Error, sync::Arc};

use async_trait::async_trait;
use langchain_rust::tools::Tool as LangchainTool;
use ollama_rs::generation::functions::tools::Tool as OllamaTool;
use serde_json::Value;

// we cant implement for OllamaTool as its out-of-crate,
// so we use a wrapper tuple-struct
pub struct LangchainToolCompat(Arc<dyn OllamaTool>);

impl LangchainToolCompat {
    pub fn new(tool: Arc<dyn OllamaTool>) -> Self {
        LangchainToolCompat(tool)
    }
}

#[async_trait]
impl LangchainTool for LangchainToolCompat {
    fn name(&self) -> String {
        self.0.name()
    }

    fn description(&self) -> String {
        self.0.description()
    }

    fn parameters(&self) -> serde_json::Value {
        self.0.parameters()
    }

    async fn call(&self, input: &str) -> Result<String, Box<dyn Error>> {
        self.0.call(input).await
    }

    async fn run(&self, input: Value) -> Result<String, Box<dyn Error>> {
        self.0.run(input).await
    }

    async fn parse_input(&self, input: &str) -> Value {
        self.0.parse_input(input).await
    }
}
