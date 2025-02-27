use crate::program::atomics::MessageInput;
use ollama_rs::{
    error::OllamaError, generation::functions::tools::Tool,
    generation::functions::OpenAIFunctionCall,
};
use openai_dive::v1::api::Client;
use openai_dive::v1::resources::chat::*;
use serde_json::{json, Value};
use std::sync::Arc;

pub struct OpenAIExecutor {
    model: String,
    client: Client,
}

impl OpenAIExecutor {
    pub fn new(model: String, api_key: String) -> Self {
        Self {
            model,
            client: Client::new(api_key),
        }
    }

    pub async fn generate_text(
        &self,
        input: Vec<MessageInput>,
        schema: Option<&String>,
    ) -> Result<String, OllamaError> {
        let messages: Vec<ChatMessage> = input
            .into_iter()
            .map(|msg| match msg.role.as_str() {
                "user" => ChatMessage::User {
                    content: ChatMessageContent::Text(msg.content),
                    name: None,
                },
                "assistant" => ChatMessage::Assistant {
                    content: Some(ChatMessageContent::Text(msg.content)),
                    tool_calls: None,
                    name: None,
                    refusal: None, // Add this line
                },
                "system" => ChatMessage::System {
                    content: ChatMessageContent::Text(msg.content),
                    name: None,
                },
                _ => ChatMessage::User {
                    // Default to user if role is unknown
                    content: ChatMessageContent::Text(msg.content),
                    name: None,
                },
            })
            .collect();

        let parameters = if let Some(schema_str) = schema {
            // Parse the schema string into a Value
            let mut schema_json: Value = serde_json::from_str(schema_str)
                .map_err(|e| OllamaError::from(format!("Invalid schema JSON: {:?}", e)))?;

            if let Value::Object(ref mut map) = schema_json {
                map.insert("additionalProperties".to_string(), Value::Bool(false));
            }

            ChatCompletionParametersBuilder::default()
                .model(self.model.clone())
                .messages(messages)
                .response_format(ChatCompletionResponseFormat::JsonSchema(
                    JsonSchemaBuilder::default()
                        .name("structured_output")
                        .schema(schema_json)
                        .strict(true)
                        .build()
                        .map_err(|e| {
                            OllamaError::from(format!("Could not build JSON schema: {:?}", e))
                        })?,
                ))
                .build()
        } else {
            ChatCompletionParametersBuilder::default()
                .model(self.model.clone())
                .messages(messages)
                .response_format(ChatCompletionResponseFormat::Text)
                .build()
        }
        .map_err(|e| OllamaError::from(format!("Could not build message parameters: {:?}", e)))?;

        let result = self.client.chat().create(parameters).await.map_err(|e| {
            OllamaError::from(format!("Failed to parse OpenAI API response: {:?}", e))
        })?;

        let message = match &result.choices[0].message {
            ChatMessage::Assistant { content, .. } => {
                if let Some(ChatMessageContent::Text(text)) = content {
                    text.clone()
                } else {
                    return Err(OllamaError::from(
                        "Unexpected message content format".to_string(),
                    ));
                }
            }
            _ => return Err(OllamaError::from("Unexpected message type".to_string())),
        };

        Ok(message)
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
            .map(|tool| ChatCompletionTool {
                r#type: ChatCompletionToolType::Function,
                function: ChatCompletionFunction {
                    name: tool.name().to_lowercase().replace(' ', "_"),
                    description: Some(tool.description()),
                    parameters: tool.parameters(),
                },
            })
            .collect();

        let messages = vec![ChatMessage::User {
            content: ChatMessageContent::Text(prompt.to_string()),
            name: None,
        }];

        let parameters = ChatCompletionParametersBuilder::default()
            .model(self.model.clone())
            .messages(messages)
            .tools(openai_tools)
            .build()
            .map_err(|e| {
                OllamaError::from(format!("Could not build message parameters: {:?}", e))
            })?;

        let result = self.client.chat().create(parameters).await.map_err(|e| {
            OllamaError::from(format!("Failed to parse OpenAI API response: {:?}", e))
        })?;
        let message = result.choices[0].message.clone();

        if raw_mode {
            self.handle_raw_mode(message)
        } else {
            self.handle_normal_mode(message, tools, oai_parser).await
        }
    }

    fn handle_raw_mode(&self, message: ChatMessage) -> Result<String, OllamaError> {
        let mut raw_calls = Vec::new();

        if let ChatMessage::Assistant {
            tool_calls: Some(tool_calls),
            ..
        } = message
        {
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
        message: ChatMessage,
        tools: Vec<Arc<dyn Tool>>,
        oai_parser: Arc<OpenAIFunctionCall>,
    ) -> Result<String, OllamaError> {
        let mut results = Vec::<String>::new();

        if let ChatMessage::Assistant {
            tool_calls: Some(tool_calls),
            ..
        } = message
        {
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
