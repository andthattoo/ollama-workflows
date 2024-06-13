//Reseverd keywords
use std::collections::HashMap;
use std::fmt;

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

#[derive(Debug, serde::Deserialize, Clone)]
pub struct CustomToolTemplate {
    pub name: String,
    pub description: String,
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: HashMap<String, String>,
}

/// Configuration for the workflow
#[derive(Debug, serde::Deserialize)]
pub struct Config {
    /// Maximum number of steps to execute. Program halts afterwards.
    pub max_steps: u32,
    /// Maximum execution time in seconds. Program halts afterwards.
    pub max_time: u64,
    /// Set of tools to use in the workflow
    pub tools: Vec<String>,
    /// A custom tool that user can define within workflow. 
    pub custom_tool: Option<CustomToolTemplate>,
    /// Maximum number of tokens for LLMs to generate per run.
    pub max_tokens: Option<i32>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Input {
    pub name: String,
    pub value: InputValue,
    pub required: bool,
}

#[derive(Debug, serde::Deserialize)]
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

#[derive(Debug, serde::Deserialize)]
pub struct SearchQuery {
    #[serde(rename = "type")]
    pub value_type: InputValueType,
    pub key: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct InputValue {
    #[serde(rename = "type")]
    pub value_type: InputValueType,
    pub index: Option<usize>,
    pub search_query: Option<SearchQuery>,
    pub key: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputType {
    Write,
    Insert,
    Push,
}

#[derive(Debug, serde::Deserialize)]
pub struct Output {
    #[serde(rename = "type")]
    pub output_type: OutputType,
    pub key: String,
    pub value: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Operator {
    Generation,
    FunctionCalling,
    Check,
    Search,
    End,
}

#[derive(Debug, serde::Deserialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub description: String,
    pub prompt: String,
    pub inputs: Vec<Input>,
    pub operator: Operator,
    pub outputs: Vec<Output>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Edge {
    pub source: String,
    pub target: String,
    pub condition: Option<Condition>,
    pub fallback: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub enum Expression {
    Equal,
    NotEqual,
    Contains,
    NotContains,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

impl Expression {
    pub fn evaluate(&self, input: &str, expected: &str) -> bool {
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
        }
    }
}

#[derive(Debug, serde::Deserialize)]
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
#[derive(Debug, Clone)]
pub enum Model {
    // Ollama models
    /// Nous's Hermes-2-Theta model, q8_0 quantized
    NousTheta,
    /// Microsoft's Phi3 Medium model, q4_1 quantized
    Phi3Medium,
    /// Microsoft's Phi3 Medium model, 128k context length, q4_1 quantized
    Phi3Medium128k,
    /// Microsoft's Phi3 Mini model, 3.8b parameters
    Phi3Mini,
    // OpenAI models
    /// OpenAI's GPT-3.5 Turbo model
    GPT3_5Turbo,
    /// OpenAI's GPT-4 Turbo model
    GPT4Turbo,
    /// OpenAI's GPT-4o model
    GPT4o,
}

#[derive(Debug, Clone)]
pub enum ModelProvider {
    Ollama,
    OpenAI,
}

impl From<Model> for ModelProvider {
    fn from(model: Model) -> Self {
        match model {
            // Ollama models
            Model::NousTheta => ModelProvider::Ollama,
            Model::Phi3Medium => ModelProvider::Ollama,
            Model::Phi3Medium128k => ModelProvider::Ollama,
            Model::Phi3Mini => ModelProvider::Ollama,
            // OpenAI models
            Model::GPT3_5Turbo => ModelProvider::OpenAI,
            Model::GPT4Turbo => ModelProvider::OpenAI,
            Model::GPT4o => ModelProvider::OpenAI,
        }
    }
}

//implement display with mathcing
impl fmt::Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Ollama models
            Model::NousTheta => write!(f, "adrienbrault/nous-hermes2theta-llama3-8b:q8_0"),
            Model::Phi3Medium => write!(f, "phi3:14b-medium-4k-instruct-q4_1"),
            Model::Phi3Medium128k => write!(f, "phi3:14b-medium-128k-instruct-q4_1"),
            Model::Phi3Mini => write!(f, "phi3:3.8b"),
            // OpenAI models
            Model::GPT3_5Turbo => write!(f, "gpt-3.5-turbo"),
            Model::GPT4Turbo => write!(f, "gpt-4-turbo"),
            Model::GPT4o => write!(f, "gpt-4o"),
        }
    }
}
