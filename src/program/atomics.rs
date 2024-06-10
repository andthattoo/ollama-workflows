//Reseverd keywords

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

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    pub max_steps: u32,
    pub max_time: u64,
    pub tools: Vec<String>,
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
    Search,
    Pop,
    Peek,
    GetAll,
    String,
}

#[derive(Debug, serde::Deserialize)]
pub struct InputValue {
    #[serde(rename = "type")]
    pub value_type: InputValueType,
    pub index: Option<usize>,
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
    Condition,
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
    pub target_if_not_met: Option<String>,
    pub condition: Option<Condition>,
    pub fallback: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub enum Expression{
    Equal,
    NotEqual,
    Contains,
    NotContains,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

impl Expression{
    pub fn evaluate(&self, input: &str, expected: &str) -> bool{
        match self{
            Expression::Equal => input == expected,
            Expression::NotEqual => input != expected,
            Expression::Contains => input.contains(expected),
            Expression::NotContains => !input.contains(expected),
            Expression::GreaterThan => input.parse::<f64>().unwrap() > expected.parse::<f64>().unwrap(),
            Expression::LessThan => input.parse::<f64>().unwrap() < expected.parse::<f64>().unwrap(),
            Expression::GreaterThanOrEqual => input.parse::<f64>().unwrap() >= expected.parse::<f64>().unwrap(),
            Expression::LessThanOrEqual => input.parse::<f64>().unwrap() <= expected.parse::<f64>().unwrap(),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Condition {
    pub input: InputValue,
    pub expected: String,
    pub expression: Expression,
}

pub enum Model {
    NousTheta,
    Phi3Medium,
    Phi3Medium128k,
    Phi3Mini,
}

impl Model {
    pub fn to_string(&self) -> String {
        match self {
            Model::NousTheta => "adrienbrault/nous-hermes2theta-llama3-8b:q8_0".to_string(),
            Model::Phi3Medium => "phi3:14b-medium-4k-instruct-q4_1".to_string(),
            Model::Phi3Medium128k => "phi3:14b-medium-128k-instruct-q4_1".to_string(),
            Model::Phi3Mini => "Phi3Mini".to_string(),
        }
    }
}
