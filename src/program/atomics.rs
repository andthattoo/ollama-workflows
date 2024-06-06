
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
    Read,
    Search,
    Pop,
    Peek,
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