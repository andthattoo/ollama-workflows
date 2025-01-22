use crate::program::io::{Input, InputValue, Output};
use crate::ProgramMemory;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

pub static R_INPUT: &str = "__input";
pub static R_OUTPUT: &str = "__result";
pub static R_END: &str = "__end";

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
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum CustomToolModeTemplate {
    Custom {
        parameters: Value,
    },
    HttpRequest {
        url: String,
        method: String,
        #[serde(default)]
        headers: Option<HashMap<String, String>>,
        #[serde(default)]
        body: Option<HashMap<String, String>>,
    },
}

#[derive(Debug, Deserialize, Clone)]
pub struct CustomToolTemplate {
    pub name: String,
    pub description: String,
    #[serde(flatten)]
    pub mode: CustomToolModeTemplate,
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
    /// A list of custom tools that user can define within workflow.
    pub custom_tools: Option<Vec<CustomToolTemplate>>,
    /// Maximum number of tokens for LLMs to generate per run.
    pub max_tokens: Option<i32>,
    pub temperature: Option<f64>,   // Add temperature field
    pub top_k: Option<i32>,         // Add top_k field
    pub logits: Option<bool>,       // Add logits field
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Operator {
    Generation,
    FunctionCalling,
    FunctionCallingRaw,
    Search,
    Sample,
    End,
}

/// A message entry.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageInput {
    /// Role, usually `user`, `assistant` or `system`.
    pub role: String,
    /// Message content.
    pub content: String,
}

impl MessageInput {
    /// Creates a new message with the given content for `assistant` role.
    pub fn new_assistant_message(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }

    /// Creates a new message with the given content for `user` role.
    pub fn new_user_message(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Task {
    /// A unique identifier for the task
    pub id: String,
    /// A human-readable name for the task
    #[serde(default)]
    pub name: String,
    /// A description of the task
    #[serde(default)]
    pub description: String,
    /// Prompt of the task. Can have placeholders for inputs e.g. {query}.
    pub messages: Vec<MessageInput>,
    /// The operator to be used for the task
    pub operator: Operator,
    /// Inputs of the task, defaults to empty list if omitted (i.e. no inputs).
    #[serde(default)]
    pub inputs: Vec<Input>,
    /// Outputs of the task, defaults to empty list if omitted (i.e. no outputs).
    #[serde(default)]
    pub outputs: Vec<Output>,
    /// Schema for structured outputs.
    pub schema: Option<String>,
}

impl Task {
    /// Creates a new chat history entry with the given content for `assistant` role.
    pub fn append_assistant_message(&mut self, content: impl Into<String>) {
        self.messages.push(MessageInput {
            role: "assistant".to_string(),
            content: content.into(),
        });
    }

    /// Creates a new chat history entry with the given content for `user` role.
    pub fn append_user_message(&mut self, content: impl Into<String>) {
        self.messages.push(MessageInput {
            role: "user".to_string(),
            content: content.into(),
        });
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum TaskOutputInput {
    Single(InputValue),
    Multiple(Vec<InputValue>),
}

#[derive(Debug, Deserialize)]
pub struct TaskOutput {
    pub input: TaskOutputInput,
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

#[derive(Debug, Deserialize)]
pub struct Condition {
    pub input: InputValue,
    pub expected: String,
    pub expression: Expression,
    pub target_if_not: String,
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
