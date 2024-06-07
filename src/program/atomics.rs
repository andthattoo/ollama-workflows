//Reseverd keywords

pub static R_INPUT: &str = "__input";
pub static R_OUTPUT: &str = "__result";
pub static R_OPERATOR: &str = "operator";
pub static R_TASKS: &str = "tasks";
pub static R_CONFIG: &str = "config";
pub static R_EXPECTED: &str = "__expected";
pub static R_OUTPUTS: &str = "__output";
pub static RESERVED_KEYWORDS: [&str; 7] = [
    R_INPUT, R_OUTPUT, R_OPERATOR, R_TASKS, R_CONFIG, R_EXPECTED, R_OUTPUTS,
];

pub fn in_reserved_keywords(s: &str) -> bool {
    RESERVED_KEYWORDS.contains(&s)
}

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    pub max_steps: u32,
    pub max_time: u64,
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
    FuzzyCheck,
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
    pub fallback: Option<String>,
}

pub enum Model {
    NousTheta,
    Phi3Medium,
    Phi3Mini,
}

impl Model {
    pub fn to_string(&self) -> String {
        match self {
            Model::NousTheta => "NousTheta".to_string(),
            Model::Phi3Medium => "Phi3Medium".to_string(),
            Model::Phi3Mini => "Phi3Mini".to_string(),
        }
    }
}