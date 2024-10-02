use crate::program::atomics::{CustomToolModeTemplate, CustomToolTemplate};
use async_trait::async_trait;
use ollama_rs::generation::functions::tools::Tool;
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use std::error::Error;

pub enum CustomToolMode {
    Custom {
        parameters: Value,
    },
    HttpRequest {
        url: String,
        method: String,
        headers: HashMap<String, String>,
        body: HashMap<String, String>,
    },
}

pub struct CustomTool {
    pub name: String,
    pub description: String,
    pub mode: CustomToolMode,
}

impl CustomTool {
    pub fn new_from_template(template: CustomToolTemplate) -> Self {
        CustomTool {
            name: template.name,
            description: template.description,
            mode: match template.mode {
                CustomToolModeTemplate::Custom { parameters } => {
                    CustomToolMode::Custom { parameters }
                }
                CustomToolModeTemplate::HttpRequest {
                    url,
                    method,
                    headers,
                    body,
                } => CustomToolMode::HttpRequest {
                    url,
                    method,
                    headers: headers.unwrap_or_default(),
                    body: body.unwrap_or_default(),
                },
            },
        }
    }
}

#[async_trait]
impl Tool for CustomTool {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn description(&self) -> String {
        self.description.clone()
    }

    fn parameters(&self) -> Value {
        match &self.mode {
            CustomToolMode::Custom { parameters } => parameters.clone(),
            CustomToolMode::HttpRequest { body, .. } => {
                let properties: HashMap<_, _> = body.iter().map(|(k, _)| {
                    (k.clone(), json!({ "type": "string", "description": format!("The value for {}", k) }))
                }).collect();

                json!({
                    "type": "object",
                    "properties": properties,
                    "required": body.keys().collect::<Vec<_>>()
                })
            }
        }
    }

    async fn run(&self, input: Value) -> Result<String, Box<dyn Error>> {
        match &self.mode {
            CustomToolMode::Custom { .. } => {
                Err("Custom mode can't execute tools, use function_calling_raw".into())
            }
            CustomToolMode::HttpRequest {
                url,
                method,
                headers,
                body: _,
            } => {
                let client = Client::new();
                let mut request = match method.as_str() {
                    "GET" => client.get(url),
                    "POST" => client.post(url),
                    "PUT" => client.put(url),
                    "DELETE" => client.delete(url),
                    _ => return Err("Unsupported HTTP method".into()),
                };

                for (key, value) in headers {
                    request = request.header(key, value);
                }

                if let Ok(token) = env::var("API_KEY") {
                    request = request.header("Authorization", format!("Bearer {}", token));
                }

                if method != "GET" {
                    let body: HashMap<String, String> = input
                        .as_object()
                        .unwrap()
                        .iter()
                        .map(|(k, v)| (k.clone(), v.as_str().unwrap().to_string()))
                        .collect();
                    request = request.json(&body);
                }

                let response = request.send().await?;
                let result = response.text().await?;
                Ok(result)
            }
        }
    }
}
