use std::sync::Arc;

use crate::program::atomics::MessageInput;
use ollama_rs::error::OllamaError;
use ollama_rs::{generation::functions::tools::Tool, generation::functions::OpenAIFunctionCall};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// [Reasoning](https://openrouter.ai/docs/use-cases/reasoning-tokens) setting for [`OpenRouterRequest`].
#[derive(Debug, Serialize)]
struct OpenRouterReasoning {
    /// Maximum number of tokens to use for reasoning, Anthropic style.
    max_tokens: u64,
}

impl Default for OpenRouterReasoning {
    fn default() -> Self {
        Self { max_tokens: 2000 }
    }
}

#[derive(Debug, Serialize)]
struct OpenRouterRequest {
    model: String,
    messages: Vec<OpenRouterMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<OpenRouterTool>>,
    /// If `Some`, the model will return reasoning data.
    reasoning: Option<OpenRouterReasoning>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenRouterTool {
    r#type: String,
    function: OpenRouterFunction,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenRouterFunction {
    name: String,
    description: Option<String>,
    parameters: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct OpenRouterMessage {
    role: String,
    content: Option<String>,   // `Option` since it can be null
    reasoning: Option<String>, // returned with reasoning models like DeepSeek-R1
    refusal: Option<String>,
    tool_calls: Option<Vec<OpenRouterToolCall>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct OpenRouterToolCall {
    id: String,
    r#type: String,
    function: OpenRouterToolCallFunction,
    index: i32, // Added this field
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct OpenRouterToolCallFunction {
    name: String,
    arguments: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct OpenRouterResponse {
    id: String,
    provider: String,
    model: String,
    object: String,
    created: i64,
    choices: Vec<OpenRouterChoice>,
    usage: OpenRouterUsage,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct OpenRouterChoice {
    index: i32,
    finish_reason: Option<String>,
    message: OpenRouterMessage,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct OpenRouterUsage {
    prompt_tokens: i32,
    completion_tokens: i32,
    total_tokens: i32,
}

pub struct OpenRouterExecutor {
    model: String,
    api_key: String,
    client: Client,
}

impl OpenRouterExecutor {
    pub fn new(model: String, api_key: String) -> Self {
        Self {
            model,
            api_key,
            client: Client::new(),
        }
    }

    pub async fn generate_text(
        &self,
        input: Vec<MessageInput>,
        _schema: Option<&String>,
        with_reasoning: bool,
    ) -> Result<String, OllamaError> {
        let messages: Vec<OpenRouterMessage> = input
            .into_iter()
            .map(|msg| OpenRouterMessage {
                role: msg.role,
                content: Some(msg.content),
                reasoning: None,
                refusal: None,
                tool_calls: None,
            })
            .collect();

        let request = OpenRouterRequest {
            model: self.model.clone(),
            messages,
            tools: None,
            reasoning: if with_reasoning {
                Some(OpenRouterReasoning::default())
            } else {
                None
            },
        };

        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Authorization",
            header::HeaderValue::from_str(&format!("Bearer {}", self.api_key))
                .map_err(|e| OllamaError::from(format!("Invalid header value: {}", e)))?,
        );
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        let response = self
            .client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .headers(headers)
            .json(&request)
            .send()
            .await
            .map_err(|e| OllamaError::from(format!("Failed to send request: {}", e)))?;

        let response_body: OpenRouterResponse = response
            .json()
            .await
            .map_err(|e| OllamaError::from(format!("Failed to parse response: {}", e)))?;

        if let Some(choice) = response_body.choices.first() {
            let content = choice
                .message
                .content
                .as_ref()
                .ok_or_else(|| OllamaError::from("No content in response".to_string()))
                .map(|s| s.to_string())?;

            match choice.message.reasoning.as_ref().map(|s| s.to_string()) {
                // if there is a reasoning, return it with the `think` tags, followed by the content
                Some(reasoning) => Ok(format!("<think>\n{}\n</think>\n\n{}", reasoning, content)),
                // otherwise just return the content
                None => Ok(content),
            }
        } else {
            Err(OllamaError::from("No response generated".to_string()))
        }
    }

    pub async fn function_call(
        &self,
        prompt: &str,
        tools: Vec<Arc<dyn Tool>>,
        raw_mode: bool,
        oai_parser: Arc<OpenAIFunctionCall>,
    ) -> Result<String, OllamaError> {
        let openai_tools: Vec<_> = tools
            .iter()
            .map(|tool| OpenRouterTool {
                r#type: "function".to_string(),
                function: OpenRouterFunction {
                    name: tool.name().to_lowercase().replace(' ', "_"),
                    description: Some(tool.description()),
                    parameters: tool.parameters(),
                },
            })
            .collect();

        let messages = vec![OpenRouterMessage {
            role: "user".to_string(),
            content: Some(prompt.to_string()),
            refusal: None,
            reasoning: None, // we dont make use of returned reasoning data
            tool_calls: None,
        }];

        let request = OpenRouterRequest {
            model: self.model.clone(),
            messages,
            tools: Some(openai_tools),
            reasoning: None,
        };

        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Authorization",
            header::HeaderValue::from_str(&format!("Bearer {}", self.api_key))
                .map_err(|e| OllamaError::from(format!("Invalid header value: {}", e)))?,
        );
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        let response = self
            .client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .headers(headers)
            .json(&request)
            .send()
            .await
            .map_err(|e| OllamaError::from(format!("Failed to send request: {}", e)))?;

        let response_text = response
            .text()
            .await
            .map_err(|e| OllamaError::from(format!("Failed to get response text: {}", e)))?;

        let response_body: OpenRouterResponse =
            serde_json::from_str(&response_text).map_err(|e| {
                OllamaError::from(format!(
                    "Failed to parse response: {} for text: {}",
                    e, response_text
                ))
            })?;

        if raw_mode {
            self.handle_raw_mode(response_body.choices[0].message.clone())
        } else {
            self.handle_normal_mode(response_body.choices[0].message.clone(), tools, oai_parser)
                .await
        }
    }

    fn handle_raw_mode(&self, message: OpenRouterMessage) -> Result<String, OllamaError> {
        let mut raw_calls = Vec::new();

        if let Some(tool_calls) = message.tool_calls {
            for tool_call in tool_calls {
                let call_json = json!({
                    "name": tool_call.function.name,
                    "arguments": serde_json::from_str::<serde_json::Value>(&tool_call.function.arguments)?
                });
                raw_calls.push(serde_json::to_string(&call_json)?);
            }
        }

        Ok(raw_calls.join("\n\n"))
    }

    async fn handle_normal_mode(
        &self,
        message: OpenRouterMessage,
        tools: Vec<Arc<dyn Tool>>,
        oai_parser: Arc<OpenAIFunctionCall>,
    ) -> Result<String, OllamaError> {
        let mut results = Vec::<String>::new();

        if let Some(tool_calls) = message.tool_calls {
            for tool_call in tool_calls {
                for tool in &tools {
                    if tool.name().to_lowercase().replace(' ', "_") == tool_call.function.name {
                        let tool_params: Value =
                            serde_json::from_str(&tool_call.function.arguments)?;
                        let res = oai_parser
                            .function_call_with_history(
                                tool_call.function.name.clone(),
                                tool_params,
                                tool.clone(),
                            )
                            .await;
                        match res {
                            Ok(result) => results.push(result.message.unwrap().content),
                            Err(e) => {
                                return Err(OllamaError::from(format!(
                                    "Could not generate text: {:?}",
                                    e
                                )))
                            }
                        }
                    }
                }
            }
        }

        Ok(results.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use crate::{Model, ModelProvider};

    use super::*;

    /// Run with:
    ///
    /// ```sh
    /// OPENROUTER_API_KEY=$API_KEY_HERE cargo test --package ollama-workflows --lib -- api_interface::open_router::tests::test_reasoning --exact --show-output --ignored
    /// ```
    #[tokio::test]
    #[ignore = "requires OpenRouter key"]
    async fn test_reasoning() {
        let api_key = std::env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY not set");

        let model = Model::ORR1;
        assert_eq!(model.provider(), ModelProvider::OpenRouter);
        assert!(model.has_reasoning());

        let executor = OpenRouterExecutor::new(model.to_string(), api_key);

        let result = executor
            .generate_text(
                vec![MessageInput::new_user_message("Hi!")],
                None,
                model.has_reasoning(),
            )
            .await
            .unwrap();

        println!("{}", result);
        assert!(result.contains("<think>") && result.contains("</think>"));
    }
}
