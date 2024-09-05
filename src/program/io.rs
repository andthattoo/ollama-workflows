use serde::Deserialize;

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
