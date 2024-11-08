use log::warn;
use ollama_rs::{
    error::OllamaError, generation::functions::tools::Tool,
    generation::functions::OpenAIFunctionCall,
};
use reqwest::Client;
use serde_json::{json, Value};
use std::{error::Error, sync::Arc};

pub struct GeminiExecutor {
    model: String,
    api_key: String,
    client: Client,
    max_tokens: i32,
}

impl GeminiExecutor {
    pub fn new(model: String, api_key: String, max_tokens: i32) -> Self {
        Self {
            model,
            api_key,
            client: Client::new(),
            max_tokens,
        }
    }

    // now supports structured output
    pub async fn generate_text(
        &self,
        prompt: &str,
        schema: &Option<String>,
    ) -> Result<String, OllamaError> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let mut generation_config = json!({
            "temperature": 1.0,
            "maxOutputTokens": self.max_tokens,
            "topP": 0.8,
            "topK": 10
        });

        // If schema is provided, add structured output configuration
        if let Some(schema_str) = schema {
            let schema_json: Value = serde_json::from_str(schema_str)
                .map_err(|e| OllamaError::from(format!("Invalid schema JSON: {:?}", e)))?;

            generation_config.as_object_mut().unwrap().extend(
                json!({
                    "response_mime_type": "application/json",
                    "response_schema": schema_json
                })
                .as_object()
                .unwrap()
                .clone(),
            );
        }

        let body = json!({
            "contents": [{
                "parts": [
                    {"text": prompt}
                ]
            }],
            "safetySettings": [
                {
                    "category": "HARM_CATEGORY_DANGEROUS_CONTENT",
                    "threshold": "BLOCK_ONLY_HIGH"
                }
            ],
            "generationConfig": generation_config
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                OllamaError::from(format!("Gemini API request failed: {:?}", e.source()))
            })?;

        // check status
        if let Err(e) = response.error_for_status_ref() {
            return Err(OllamaError::from(format!(
                "Gemini API request failed with status {}: {:?}",
                response.status(),
                e.source()
            )));
        }

        let response_body: Value = response.json().await.map_err(|e| {
            OllamaError::from(format!("Failed to parse Gemini API response: {}", e))
        })?;

        self.extract_generated_text(response_body)
    }

    fn extract_generated_text(&self, response: Value) -> Result<String, OllamaError> {
        warn!(
            "Full Gemini Response: {}",
            serde_json::to_string_pretty(&response).unwrap()
        );

        response["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| {
                OllamaError::from("Failed to extract generated text from response".to_string())
            })
    }

    fn extract_tools(&self, response: Value) -> Result<Value, OllamaError> {
        let candidate = response["candidates"]
            .get(0)
            .ok_or_else(|| OllamaError::from("No candidates found in response".to_string()))?;

        let content = &candidate["content"]["parts"][0];

        if let Some(function_call) = content.get("functionCall") {
            Ok(function_call.clone())
        } else if let Some(text) = content.get("text") {
            Ok(json!({"text": text}))
        } else {
            Err(OllamaError::from("Unexpected response format".to_string()))
        }
    }

    pub async fn function_call(
        &self,
        prompt: &str,
        tools: Vec<Arc<dyn Tool>>,
        raw_mode: bool,
        oai_parser: Arc<OpenAIFunctionCall>,
    ) -> Result<String, OllamaError> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let function_declarations: Vec<Value> = tools
            .iter()
            .map(|tool| {
                json!({
                    "name": tool.name(),
                    "description": tool.description(),
                    "parameters": tool.parameters()
                })
            })
            .collect();

        let body = json!({
            "system_instruction": {
                "parts": {
                    "text": "You are a helpful function calling assistant."
                }
            },
            "tools": {"function_declarations" : function_declarations},
            "tool_config": {
                "function_calling_config": {"mode": "ANY"}
            },
            "contents": {
                "role": "user",
                "parts": {
                    "text": prompt
                }
            }
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                OllamaError::from(format!("Gemini API request failed: {:?}", e.source()))
            })?;

        // check status
        if let Err(e) = response.error_for_status_ref() {
            return Err(OllamaError::from(format!(
                "Gemini API request failed with status {}: {:?}",
                response.status(),
                e.source()
            )));
        }

        let response_body: Value = response.json().await.map_err(|e| {
            OllamaError::from(format!("Failed to parse Gemini API response: {:?}", e))
        })?;

        let tool_call = self.extract_tools(response_body)?;

        for tool in &tools {
            if tool.name().to_lowercase().replace(' ', "_")
                == tool_call["name"].as_str().unwrap_or("")
            {
                if raw_mode {
                    let raw_result = serde_json::to_string(&tool_call);
                    return match raw_result {
                        Ok(raw_call) => Ok(raw_call),
                        Err(e) => Err(OllamaError::from(format!(
                            "Raw Call string conversion failed {:?}",
                            e
                        ))),
                    };
                }
                let res = oai_parser
                    .function_call_with_history(
                        tool_call["name"].as_str().unwrap_or("").to_string(),
                        tool_call["args"].clone(),
                        tool.clone(),
                    )
                    .await;
                return match res {
                    Ok(result) => Ok(result.message.unwrap().content),
                    Err(e) => Err(OllamaError::from(format!(
                        "Could not generate text: {:?}",
                        e
                    ))),
                };
            }
        }

        Err(OllamaError::from(format!(
            "No matching tool found for function: {}",
            tool_call["name"].as_str().unwrap_or("")
        )))
    }
}
