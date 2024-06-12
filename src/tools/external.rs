use crate::program::atomics::CustomToolTemplate;
use async_trait::async_trait;
use ollama_rs::generation::functions::tools::Tool;
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use std::error::Error;

pub struct CustomTool {
    pub name: String,
    pub description: String,
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: HashMap<String, String>,
}

impl CustomTool {
    pub fn new_from_template(template: CustomToolTemplate) -> Self {
        CustomTool {
            name: template.name,
            description: template.description,
            url: template.url,
            method: template.method,
            headers: template.headers,
            body: template.body,
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
        let properties: HashMap<_, _> = self
            .body
            .keys()
            .map(|k| {
                (
                    k.clone(),
                    json!({ "type": "string", "description": format!("The value for {}", k) }),
                )
            })
            .collect();

        json!({
            "type": "object",
            "properties": properties,
            "required": self.body.keys().collect::<Vec<_>>()
        })
    }

    async fn run(&self, input: Value) -> Result<String, Box<dyn Error>> {
        let client = Client::new();
        let mut request = match self.method.as_str() {
            "GET" => client.get(&self.url),
            "POST" => client.post(&self.url),
            "PUT" => client.put(&self.url),
            "DELETE" => client.delete(&self.url),
            _ => return Err(Box::from("Unsupported HTTP method")),
        };

        for (key, value) in &self.headers {
            request = request.header(key, value);
        }

        let token = env::var("API_KEY");
        if token.is_ok() {
            request = request.header("Authorization", format!("Bearer {}", token.unwrap()));
        }

        if self.method != "GET" {
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
