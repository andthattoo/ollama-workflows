use crate::program::io::{Input, InputValue, Output};
use crate::ProgramMemory;
use serde::Deserialize;
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
pub struct CustomToolTemplate {
    pub name: String,
    pub description: String,
    pub url: String,
    pub method: String,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<HashMap<String, String>>,
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
