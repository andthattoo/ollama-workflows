use ollama_rs::models::LocalModel;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Enum for the models that can be used in the workflow
/// Import the model to executor using:
/// ```rust
/// let exe = Executor::new(Model::Phi3Medium);
/// ```
/// These models are selected based on their performance and size.
/// You can add models by creating a pull request.
#[derive(Default, Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Model {
    // Ollama models
    /// [Nous's Hermes-2-Theta model](https://ollama.com/finalend/hermes-3-llama-3.1:8b-q8_0), q8_0 quantized
    #[serde(rename = "finalend/hermes-3-llama-3.1:8b-q8_0")]
    NousTheta,
    /// [Microsoft's Phi3 Medium model](https://ollama.com/library/phi3:medium), q4_1 quantized
    #[serde(rename = "phi3:14b-medium-4k-instruct-q4_1")]
    Phi3Medium,
    /// [Microsoft's Phi3 Medium model, 128k context length](https://ollama.com/library/phi3:medium-128k), q4_1 quantized
    #[serde(rename = "phi3:14b-medium-128k-instruct-q4_1")]
    Phi3Medium128k,
    #[default]
    /// [Microsoft's Phi3.5 Mini model](https://ollama.com/library/phi3.5), 3.8b parameters
    #[serde(rename = "phi3.5:3.8b")]
    Phi3_5Mini,
    /// [Microsoft's Phi3.5 Mini model](https://ollama.com/library/phi3.5:3.8b-mini-instruct-fp16), 3.8b parameters
    #[serde(rename = "phi3.5:3.8b-mini-instruct-fp16")]
    Phi3_5MiniFp16,
    /// [Google's Gemma2 model](https://ollama.com/library/gemma2), 9B parameters
    #[serde(rename = "gemma2:9b-instruct-q8_0")]
    Gemma2_9B,
    /// [Google's Gemma2 model](https://ollama.com/library/gemma2), 9B parameters, fp16
    #[serde(rename = "gemma2:9b-instruct-fp16")]
    Gemma2_9BFp16,
    /// [Meta's Llama3.1 model](https://ollama.com/library/llama3.1:latest), 8B parameters
    #[serde(rename = "llama3.1:latest")]
    Llama3_1_8B,
    /// [Meta's Llama3.1 model q8](https://ollama.com/library/llama3.1:8b-text-q8_0)
    #[serde(rename = "llama3.1:8b-instruct-q8_0")]
    Llama3_1_8Bq8,
    /// [Meta's Llama3.1 model fp16](https://ollama.com/library/llama3.1:8b-instruct-fp16)
    #[serde(rename = "llama3.1:8b-instruct-fp16")]
    Llama3_1_8Bf16,
    /// 
    #[serde(rename = "llama3.1:70b-instruct-q4_0")]
    Llama3_1_70B,
    /// 
    #[serde(rename = "llama3.1:70b-instruct-q8_0")]
    Llama3_1_70Bq8,
    /// [Alibaba's Qwen2 model](https://ollama.com/library/qwen2), 7B parameters
    #[serde(rename = "qwen2.5:7b-instruct-q5_0")]
    Qwen2_5_7B,
    /// [Alibaba's Qwen2 model](https://ollama.com/library/qwen2), 7B parameters, fp16
    #[serde(rename = "qwen2.5:7b-instruct-fp16")]
    Qwen2_5_7Bf16,
    /// []
    #[serde(rename = "qwen2.5:32b-instruct-fp16")]
    Qwen2_5_32Bf16,
    // OpenAI models
    /// [OpenAI's GPT-4 Turbo model](https://platform.openai.com/docs/models/gpt-4-turbo-and-gpt-4)
    #[serde(rename = "gpt-4-turbo")]
    GPT4Turbo,
    /// [OpenAI's GPT-4o model](https://platform.openai.com/docs/models/gpt-4o)
    #[serde(rename = "gpt-4o")]
    GPT4o,
    /// [OpenAI's GPT-4o mini model](https://platform.openai.com/docs/models/gpt-4o-mini)
    #[serde(rename = "gpt-4o-mini")]
    GPT4oMini,
    /// [OpenAI's o1 mini model](https://platform.openai.com/docs/models/o1)
    #[serde(rename = "o1-mini")]
    O1Mini,
    /// [OpenAI's o1 preview model](https://platform.openai.com/docs/models/o1)
    #[serde(rename = "o1-preview")]
    O1Preview,
}

impl Model {
    pub fn supports_tool_calling(&self) -> bool {
        match self {
            // OpenAI models that support tool calling
            Model::GPT4Turbo | Model::GPT4o | Model::GPT4oMini => true,
            // Ollama models that support tool calling
            Model::Llama3_1_8B
            | Model::Llama3_1_8Bq8
            | Model::Llama3_1_8Bf16
            | Model::Phi3Medium
            | Model::Phi3Medium128k
            | Model::Gemma2_9BFp16
            | Model::Qwen2_5_7B
            | Model::Qwen2_5_7Bf16 => true,
            | Model::Qwen2_5_32Bf16 => true,
            _ => false,
        }
    }
}

impl From<Model> for String {
    fn from(model: Model) -> Self {
        model.to_string() // via Display
    }
}

impl fmt::Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // guaranteed not to fail because this is enum to string serialization
        let self_str = serde_json::to_string(&self).unwrap_or_default();

        // remove quotes from JSON
        write!(f, "{}", self_str.trim_matches('"'))
    }
}

impl TryFrom<LocalModel> for Model {
    type Error = String;
    fn try_from(value: LocalModel) -> Result<Self, Self::Error> {
        Model::try_from(value.name)
    }
}

impl TryFrom<String> for Model {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        // serde requires quotes (for JSON)
        serde_json::from_str::<Self>(&format!("\"{}\"", value))
            .map_err(|e| format!("Model {} invalid: {}", value, e))
    }
}

/// A model provider is a service that hosts the chosen Model.
/// It can be derived from the model name, e.g. GPT4o is hosted by OpenAI (via API), or Phi3 is hosted by Ollama (locally).
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum ModelProvider {
    #[serde(rename = "ollama")]
    Ollama,
    #[serde(rename = "openai")]
    OpenAI,
}

impl From<Model> for ModelProvider {
    fn from(model: Model) -> Self {
        match model {
            Model::NousTheta => ModelProvider::Ollama,
            Model::Phi3Medium => ModelProvider::Ollama,
            Model::Phi3Medium128k => ModelProvider::Ollama,
            Model::Phi3_5Mini => ModelProvider::Ollama,
            Model::Phi3_5MiniFp16 => ModelProvider::Ollama,
            Model::Llama3_1_8B => ModelProvider::Ollama,
            Model::Llama3_1_8Bq8 => ModelProvider::Ollama,
            Model::Llama3_1_8Bf16 => ModelProvider::Ollama,
            Model::Llama3_1_70B => ModelProvider::Ollama,
            Model::Llama3_1_70Bq8 => ModelProvider::Ollama,
            Model::Gemma2_9B => ModelProvider::Ollama,
            Model::Gemma2_9BFp16 => ModelProvider::Ollama,
            Model::Qwen2_5_7B => ModelProvider::Ollama,
            Model::Qwen2_5_7Bf16 => ModelProvider::Ollama,
            Model::Qwen2_5_32Bf16 => ModelProvider::Ollama,
            Model::GPT4Turbo => ModelProvider::OpenAI,
            Model::GPT4o => ModelProvider::OpenAI,
            Model::GPT4oMini => ModelProvider::OpenAI,
            Model::O1Mini => ModelProvider::OpenAI,
            Model::O1Preview => ModelProvider::OpenAI,
        }
    }
}

impl TryFrom<String> for ModelProvider {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        // serde requires quotes (for JSON)
        serde_json::from_str::<Self>(&format!("\"{}\"", value))
            .map_err(|e| format!("Model provider {} invalid: {}", value, e))
    }
}

impl fmt::Display for ModelProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // guaranteed not to fail because this is enum to string serialization
        let self_str = serde_json::to_string(&self).unwrap_or_default();

        // remove quotes from JSON
        write!(f, "{}", self_str.trim_matches('"'))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MODEL_NAME: &str = "phi3:3.8b";
    const PROVIDER_NAME: &str = "openai";
    #[test]
    fn test_model_string_conversion() {
        let model = Model::Phi3_5Mini;

        // convert to string
        let model_str: String = model.clone().into();
        assert_eq!(model_str, MODEL_NAME);

        // (try) convert from string
        let model_from = Model::try_from(model_str).expect("should convert");
        assert_eq!(model_from, model);

        // (try) convert from string
        let model = Model::try_from("this-model-does-not-will-not-exist".to_string());
        assert!(model.is_err());
    }

    #[test]
    fn test_model_string_serde() {
        let model = Model::Phi3_5Mini;

        // serialize to string via serde
        let model_str = serde_json::to_string(&model).expect("should serialize");
        assert_eq!(model_str, format!("\"{}\"", MODEL_NAME));

        // deserialize from string via serde
        let model_from: Model = serde_json::from_str(&model_str).expect("should deserialize");
        assert_eq!(model_from, model);

        // (try) deserialize from invalid model
        let bad_model = serde_json::from_str::<Model>("\"this-model-does-not-will-not-exist\"");
        assert!(bad_model.is_err());
    }

    #[test]
    fn test_provider_string_serde() {
        let provider = ModelProvider::OpenAI;

        // serialize to string via serde
        let provider_str = serde_json::to_string(&provider).expect("should serialize");
        assert_eq!(provider_str, format!("\"{}\"", PROVIDER_NAME));

        // deserialize from string via serde
        let provider_from: ModelProvider =
            serde_json::from_str(&provider_str).expect("should deserialize");
        assert_eq!(provider_from, provider);

        // (try) deserialize from invalid model
        let bad_provider =
            serde_json::from_str::<ModelProvider>("\"this-provider-does-not-will-not-exist\"");
        assert!(bad_provider.is_err());
    }
}
