use std::collections::HashMap;
use std::fmt;

use ollama_rs::models::LocalModel;
use serde::{Deserialize, Serialize};

use crate::ProgramMemory;

pub static R_INPUT: &str = "__input";
pub static R_OUTPUT: &str = "__result";
pub static R_END: &str = "__end";
pub static R_EXPECTED: &str = "__expected";
pub static R_OUTPUTS: &str = "__output";

pub static TOOLS: [&str; 6] = [
    "browserless",
    "jina",
    "serper",
    "duckduckgo",
    "stock",
    "scraper",
];

pub fn in_tools(tools: &Vec<String>) -> bool {
    for tool in tools {
        if !TOOLS.contains(&tool.as_str()) {
            return false;
        }
    }
    true
}

#[derive(Debug, Deserialize, Clone)]
pub struct CustomToolTemplate {
    pub name: String,
    pub description: String,
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: HashMap<String, String>,
}

/// Configuration for the workflow
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Maximum number of steps to execute. Program halts afterwards.
    pub max_steps: u32,
    /// Maximum execution time in seconds. Program halts afterwards.
    pub max_time: u64,
    /// Set of tools to use in the workflow
    #[serde(default)]
    pub tools: Vec<String>,
    /// A custom tool that user can define within workflow.
    pub custom_tool: Option<CustomToolTemplate>,
    /// Maximum number of tokens for LLMs to generate per run.
    pub max_tokens: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct Input {
    pub name: String,
    pub value: InputValue,
    pub required: bool,
}

#[derive(Debug, Deserialize)]
pub struct InputValue {
    #[serde(rename = "type")]
    pub value_type: InputValueType,
    pub index: Option<usize>,
    pub search_query: Option<SearchQuery>,
    pub key: String,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InputValueType {
    Input,
    Read,
    Pop,
    Peek,
    GetAll,
    Size,
    String,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    #[serde(rename = "type")]
    pub value_type: InputValueType,
    pub key: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputType {
    Write,
    Insert,
    Push,
}

#[derive(Debug, Deserialize)]
pub struct Output {
    #[serde(rename = "type")]
    pub output_type: OutputType,
    pub key: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Operator {
    Generation,
    FunctionCalling,
    Check,
    Search,
    Sample,
    End,
}

#[derive(Debug, Deserialize)]
pub struct Task {
    /// A unique identifier for the task
    pub id: String,
    /// A human-readable name for the task
    pub name: String,
    /// A description of the task
    pub description: String,
    /// Prompt of the task. Can have placeholders for inputs e.g. {query}.
    pub prompt: String,
    #[serde(default)]
    pub inputs: Vec<Input>,
    /// The operator to be used for the task
    pub operator: Operator,
    #[serde(default)]
    pub outputs: Vec<Output>,
}

#[derive(Debug, Deserialize)]
pub struct TaskOutput {
    pub input: InputValue,
    pub to_json: Option<bool>,
    pub post_process: Option<Vec<TaskPostProcess>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TaskPostProcess {
    pub process_type: PostProcessType,
    pub lhs: Option<String>,
    pub rhs: Option<String>,
}
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum PostProcessType {
    Replace,
    Append,
    Prepend,
    Trim,
    TrimStart,
    TrimEnd,
    ToLower,
    ToUpper,
}

#[derive(Debug, Deserialize)]
pub struct Edge {
    pub source: String,
    pub target: String,
    pub condition: Option<Condition>,
    pub fallback: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum Expression {
    Equal,
    NotEqual,
    Contains,
    NotContains,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    HaveSimilar,
}

impl Expression {
    pub async fn evaluate(
        &self,
        input: &str,
        expected: &str,
        memory: Option<&mut ProgramMemory>,
    ) -> bool {
        match self {
            Expression::Equal => input == expected,
            Expression::NotEqual => input != expected,
            Expression::Contains => input.contains(expected),
            Expression::NotContains => !input.contains(expected),
            Expression::GreaterThan => {
                input.parse::<f64>().unwrap() > expected.parse::<f64>().unwrap()
            }
            Expression::LessThan => {
                input.parse::<f64>().unwrap() < expected.parse::<f64>().unwrap()
            }
            Expression::GreaterThanOrEqual => {
                input.parse::<f64>().unwrap() >= expected.parse::<f64>().unwrap()
            }
            Expression::LessThanOrEqual => {
                input.parse::<f64>().unwrap() <= expected.parse::<f64>().unwrap()
            }
            Expression::HaveSimilar => {
                let res = memory.unwrap().have_similar(expected, Some(0.95)).await;
                res.unwrap_or(false)
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Condition {
    pub input: InputValue,
    pub expected: String,
    pub expression: Expression,
    pub target_if_not: String,
}

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
    /// [Nous's Hermes-2-Theta model](https://ollama.com/adrienbrault/nous-hermes2theta-llama3-8b), q8_0 quantized
    #[serde(rename = "adrienbrault/nous-hermes2theta-llama3-8b:q8_0")]
    NousTheta,
    #[default]
    /// [Microsoft's Phi3 Medium model](https://ollama.com/library/phi3:medium), q4_1 quantized
    #[serde(rename = "phi3:14b-medium-4k-instruct-q4_1")]
    Phi3Medium,
    /// [Microsoft's Phi3 Medium model, 128k context length](https://ollama.com/library/phi3:medium-128k), q4_1 quantized
    #[serde(rename = "phi3:14b-medium-128k-instruct-q4_1")]
    Phi3Medium128k,
    /// [Microsoft's Phi3 Mini model](https://ollama.com/library/phi3:3.8b), 3.8b parameters
    #[serde(rename = "phi3:3.8b")]
    Phi3Mini,
    /// [Microsoft's Phi3.5 Mini model](https://ollama.com/library/phi3.5), 3.8b parameters
    #[serde(rename = "phi3.5:3.8b")]
    Phi3_5Mini,
    /// /// [Microsoft's Phi3.5 Mini model](https://ollama.com/library/phi3.5:3.8b-mini-instruct-fp16), 3.8b parameters
    #[serde(rename = "phi3.5:3.8b-mini-instruct-fp16")]
    Phi3_5MiniFp16,
    /// [Ollama's Llama3.1 model](https://ollama.com/library/llama3.1:latest), 8B parameters
    #[serde(rename = "llama3.1:latest")]
    Llama3_1_8B,
    // OpenAI models
    /// [OpenAI's GPT-3.5 Turbo model](https://platform.openai.com/docs/models/gpt-3-5-turbo)
    #[serde(rename = "gpt-3.5-turbo")]
    GPT3_5Turbo,
    /// [OpenAI's GPT-4 Turbo model](https://platform.openai.com/docs/models/gpt-4-turbo-and-gpt-4)
    #[serde(rename = "gpt-4-turbo")]
    GPT4Turbo,
    /// [OpenAI's GPT-4o model](https://platform.openai.com/docs/models/gpt-4o)
    #[serde(rename = "gpt-4o")]
    GPT4o,
    /// [OpenAI's GPT-4o mini model](https://platform.openai.com/docs/models/gpt-4o-mini)
    #[serde(rename = "gpt-4o-mini")]
    GPT4oMini,
}
// phi3.5:3.8b

// phi3.5:3.8b-mini-instruct-fp16

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
            Model::Phi3Mini => ModelProvider::Ollama,
            Model::Phi3_5Mini => ModelProvider::Ollama,
            Model::Phi3_5MiniFp16 => ModelProvider::Ollama,
            Model::Llama3_1_8B => ModelProvider::Ollama,
            Model::GPT3_5Turbo => ModelProvider::OpenAI,
            Model::GPT4Turbo => ModelProvider::OpenAI,
            Model::GPT4o => ModelProvider::OpenAI,
            Model::GPT4oMini => ModelProvider::OpenAI,
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
        let model = Model::Phi3Mini;

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
        let model = Model::Phi3Mini;

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
