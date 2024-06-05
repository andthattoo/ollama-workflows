
#[derive(Debug, serde::Deserialize)]
pub struct Config {
    pub max_steps: u32,
    pub max_time: u32,
}

#[derive(Debug, serde::Deserialize)]
pub struct Input {
    name: String,
    value: InputValue,
    required: bool,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputValueType {
    Read,
    // Add other input value types as needed
}

#[derive(Debug, serde::Deserialize)]
pub struct InputValue {
    #[serde(rename = "type")]
    value_type: InputValueType,
    key: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputType {
    Write,
    // Add other output types as needed
}

#[derive(Debug, serde::Deserialize)]
struct Output {
    #[serde(rename = "type")]
    output_type: OutputType,
    key: String,
    value: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Operator {
    Generation,
    // Add other operators as needed
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