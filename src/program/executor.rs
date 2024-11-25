use super::atomics::*;
use super::io::*;
use super::models::*;
use super::workflow::Workflow;
use crate::api_interface::gem_api::GeminiExecutor;
use crate::api_interface::open_router::OpenRouterExecutor;
use crate::api_interface::openai_api::OpenAIExecutor;
use crate::memory::types::Entry;
use crate::memory::{MemoryReturnType, ProgramMemory};
use crate::program::atomics::MessageInput;
use crate::program::errors::{ExecutionError, ToolError};
use crate::tools::{Browserless, CustomTool, Jina, RawDDGSearcher, RawSearchTool, SearchTool};

use rand::Rng;
use serde_json::{json, Value};
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use colored::*;

use base64::prelude::*;
use log::{debug, error, info, warn};
use rand::seq::SliceRandom;

use ollama_rs::{
    error::OllamaError,
    generation::chat::request::ChatMessageRequest,
    generation::chat::ChatMessage,
    generation::completion::request::GenerationRequest,
    generation::functions::tools::StockScraper,
    generation::functions::tools::Tool,
    generation::functions::{
        DDGSearcher, FunctionCallRequest, LlamaFunctionCall, OpenAIFunctionCall, Scraper,
    },
    generation::options::GenerationOptions,
    generation::parameters::FormatType,
    Ollama,
};

fn log_colored(msg: &str) {
    let colors = ["red", "green", "yellow", "blue", "magenta", "cyan"];

    let color = colors.choose(&mut rand::thread_rng()).unwrap();
    let colored_msg = match *color {
        "red" => msg.red(),
        "green" => msg.green(),
        "yellow" => msg.yellow(),
        "blue" => msg.blue(),
        "magenta" => msg.magenta(),
        "cyan" => msg.cyan(),
        _ => msg.green(),
    };
    warn!("{}", colored_msg);
}

/// Executor, the main struct that executes the workflow
#[derive(Default)]
pub struct Executor {
    model: Model,
    llm: Ollama,
}

impl Executor {
    /// Create a new Executor with a default Ollama instance.
    pub fn new(model: Model) -> Self {
        Self {
            model,
            llm: Ollama::default(),
        }
    }

    /// Create a new Executor for an Ollama instance at a specific host and port.
    pub fn new_at(model: Model, host: &str, port: u16) -> Self {
        Self {
            model,
            llm: Ollama::new(host, port),
        }
    }

    /// Executes the workflow
    pub async fn execute(
        &self,
        input: Option<&Entry>,
        workflow: &Workflow,
        memory: &mut ProgramMemory,
    ) -> Result<String, ExecutionError> {
        let config = workflow.get_config();
        let max_steps = config.max_steps;
        let max_time = config.max_time;

        warn!("------------------");

        if let Some(external_memory) = &workflow.external_memory {
            warn!("Reading external memory into Stack");
            memory.read_external_memory(external_memory);
        }

        warn!("Executing workflow");
        info!("Max steps: {}, Max time: {}", &max_steps, &max_time);

        if let Some(input) = input {
            memory.write(R_INPUT.to_string(), input.clone());
        }

        let mut num_steps = 0;
        let start = Instant::now();
        let mut current_step = workflow.get_step(0);

        while num_steps < max_steps && start.elapsed().as_secs() < max_time {
            if let Some(edge) = current_step {
                if edge.source == R_END {
                    warn!("Successfully completed the workflow");
                    break;
                }

                if let Some(task) = workflow.get_tasks_by_id(&edge.source) {
                    let result = self.execute_task(task, memory.borrow_mut(), config).await;

                    current_step = if result.is_ok() {
                        //if there are conditions, check them
                        if let Some(condition) = &edge.condition {
                            let value = self.handle_input(&condition.input, memory).await;
                            let eval = if condition.expression == Expression::HaveSimilar {
                                condition
                                    .expression
                                    .evaluate(
                                        &value.to_string(),
                                        &condition.expected,
                                        Some(memory.borrow_mut()),
                                    )
                                    .await
                            } else {
                                condition
                                    .expression
                                    .evaluate(&value.to_string(), &condition.expected, None)
                                    .await
                            };
                            if eval {
                                warn!(
                                    "[{}] conditions met, stepping into [{}]",
                                    &edge.source, &edge.target
                                );
                                workflow.get_step_by_id(&edge.target)
                            } else {
                                warn!(
                                    "[{}] conditions not met, stepping into [{}]",
                                    &edge.source, &condition.target_if_not
                                );
                                workflow.get_step_by_id(&condition.target_if_not)
                            }
                        } else {
                            warn!(
                                "[{}] completed successfully, stepping into [{}]",
                                &edge.source, &edge.target
                            );
                            workflow.get_step_by_id(&edge.target)
                        }
                    } else if let Some(fallback) = &edge.fallback {
                        warn!("[{}] failed, stepping into [{}]", &edge.source, &fallback);
                        error!("Task execution failed: {}", result.unwrap_err());
                        workflow.get_step_by_id(fallback)
                    } else {
                        warn!("{} failed, halting beacause of no fallback", &edge.source);
                        error!("Task execution failed");
                        return Err(ExecutionError::WorkflowFailed(format!(
                            "{:?}",
                            result.unwrap_err()
                        )));
                    };
                } else {
                    return Err(ExecutionError::WorkflowFailed(format!(
                        "Task with id [{}] not found",
                        &edge.source
                    )));
                }
            } else {
                break;
            }
            num_steps += 1;
        }
        // log if elapsed time is bigger the max time
        if start.elapsed().as_secs() >= max_time {
            warn!("Max time exceeded, halting");
            return Err(ExecutionError::WorkflowFailed(
                "Max execution time exceeded".to_string(),
            ));
        }
        // log if max steps is reached
        if num_steps >= max_steps {
            warn!("Max steps reached, halting");
            return Err(ExecutionError::WorkflowFailed(
                "Max steps reached".to_string(),
            ));
        }
        let rv = workflow.get_return_value();

        //let return_value = self.handle_input(&rv.input, memory).await;
        let return_value = match &rv.input {
            TaskOutputInput::Single(input) => self.handle_input(input, memory).await,
            TaskOutputInput::Multiple(inputs) => {
                let mut results = Vec::new();
                for input in inputs {
                    results.push(self.handle_input(input, memory).await);
                }
                MemoryReturnType::Multiple(results)
            }
        };
        let mut return_string = return_value.to_string().clone();

        if rv.to_json.is_some() && rv.to_json.unwrap() {
            let res = return_value.to_json();
            if let Some(result) = res {
                return Ok(result);
            }
        }

        if let Some(post_pr) = rv.post_process.clone() {
            for process in post_pr {
                return_string = match process.process_type {
                    PostProcessType::Replace => {
                        if process.lhs.is_none() || process.rhs.is_none() {
                            error!("lhs and rhs are required for replace post process");
                            continue;
                        }
                        return_string.replace(&process.lhs.unwrap(), &process.rhs.unwrap())
                    }
                    PostProcessType::Append => {
                        if process.rhs.is_none() {
                            error!("lhs is required for append post process");
                            continue;
                        }
                        return_string.push_str(&process.rhs.unwrap());
                        return_string
                    }
                    PostProcessType::Prepend => {
                        if process.lhs.is_none() {
                            error!("lhs is required for prepend post process");
                            continue;
                        }
                        format!("{}{}", process.lhs.unwrap(), return_string)
                    }
                    PostProcessType::ToLower => return_string.to_lowercase(),
                    PostProcessType::ToUpper => return_string.to_uppercase(),
                    PostProcessType::Trim => return_string.trim().to_string(),
                    PostProcessType::TrimStart => return_string.trim_start().to_string(),
                    PostProcessType::TrimEnd => return_string.trim_end().to_string(),
                };
            }
        }
        Ok(return_string)
    }

    async fn execute_task(
        &self,
        task: &Task,
        memory: &mut ProgramMemory,
        config: &Config,
    ) -> Result<(), ExecutionError> {
        info!("Executing task: {} with id {}", &task.name, &task.id);
        info!("Using operator: {:?}", &task.operator);

        let mut input_map: HashMap<String, MemoryReturnType> = HashMap::new();
        for input in &task.inputs {
            let value = self.handle_input(&input.value, memory).await;
            if input.required && value.is_none() {
                return Err(ExecutionError::InvalidInput);
            }
            input_map.insert(input.name.clone(), value.clone());
        }

        match task.operator {
            Operator::Generation => {
                let prompt = self.fill_prompt(&task.messages, &input_map);

                let result = self.generate_text(prompt, &task.schema, config).await;
                if result.is_err() {
                    error!("Error generating text");
                    return Err(ExecutionError::GenerationFailed(format!(
                        "{:?}",
                        result.err().unwrap()
                    )));
                }
                log_colored(
                    format!("Operator: {:?}. Output: {:?}", &task.operator, &result).as_str(),
                );
                let result_entry = Entry::try_value_or_str(&result.unwrap());
                self.handle_output(task, result_entry, memory).await;
            }
            Operator::FunctionCalling | Operator::FunctionCallingRaw => {
                let prompt = self.fill_prompt(&task.messages, &input_map);

                let raw_mode = matches!(task.operator, Operator::FunctionCallingRaw);
                let result = self.function_call(prompt, config, raw_mode).await;
                if result.is_err() {
                    error!("Error function calling");
                    return Err(ExecutionError::FunctionCallFailed(format!(
                        "{:?}",
                        result.err().unwrap()
                    )));
                }

                log_colored(
                    format!("Operator: {:?}. Output: {:?}", &task.operator, &result).as_str(),
                );
                let result_entry = Entry::try_value_or_str(&result.unwrap());
                self.handle_output(task, result_entry, memory).await;
            }
            Operator::Search => {
                let inp = self.fill_prompt(&task.messages, &input_map);
                let prompt = inp
                    .last()
                    .map(|msg| msg.content.as_str())
                    .unwrap_or_default()
                    .to_owned();

                let search_tool = RawSearchTool {};
                let search_params: Value = serde_json::from_str(&prompt).unwrap_or(json!({}));

                let query = search_params["query"].as_str().map(|s| s.to_string());
                let query = query.ok_or_else(|| {
                    ExecutionError::WebSearchFailed("Query parameter is required".to_string())
                })?;

                let search_type = search_params["search_type"].as_str().map(|s| s.to_string());
                let lang = search_params["lang"].as_str().map(|s| s.to_string());
                let n_results = search_params["n_results"].as_u64();

                let result = if let Ok(serper_key) = std::env::var("SERPER_API_KEY") {
                    if !serper_key.is_empty() {
                        search_tool
                            .search(&query, search_type.as_deref(), lang.as_deref(), n_results)
                            .await
                    } else {
                        let ddg_tool = RawDDGSearcher::new();
                        ddg_tool.search(&query, n_results.map(|n| n as usize)).await
                    }
                } else {
                    let ddg_tool = RawDDGSearcher::new();
                    ddg_tool.search(&query, n_results.map(|n| n as usize)).await
                };

                let result = result.map_err(|e| ExecutionError::WebSearchFailed(e.to_string()))?;

                log_colored(
                    format!("Operator: {:?}. Output: {:?}", &task.operator, &result).as_str(),
                );
                let result_entry = Entry::try_value_or_str(&result);
                self.handle_output(task, result_entry, memory).await;
            }
            Operator::Sample => {
                // Read Stack for each key in the inputs
                // Sample from the stack
                // fill prompts with values
                // write to memory

                let inp = self.fill_prompt(&task.messages, &input_map);
                let mut prompt = inp
                    .last()
                    .map(|msg| msg.content.as_str())
                    .unwrap_or_default()
                    .to_owned();

                for (key, value) in &input_map {
                    let v = Vec::<Entry>::from(value.clone());
                    if !v.is_empty() {
                        error!("Input for Sample operator cannot be GetAll");
                        return Err(ExecutionError::InvalidGetAllError);
                    } else {
                        let stack_lookup = value.to_string();
                        let entry = memory.get_all(&stack_lookup);
                        if entry.is_none() {
                            error!("Error sampling: {:?}", key);
                            return Err(ExecutionError::SamplingError);
                        }
                        let sample = self.sample(&entry.unwrap());
                        prompt.push_str(&format!(": {}", sample));
                    }
                }
                self.handle_output(task, Entry::try_value_or_str(&prompt), memory)
                    .await;
            }
            Operator::End => {}
        };

        Ok(())
    }

    fn fill_prompt(
        &self,
        prompt: &[MessageInput],
        input_values: &HashMap<String, MemoryReturnType>,
    ) -> Vec<MessageInput> {
        prompt
            .iter()
            .map(|message| {
                let mut filled_content = message.content.clone();
                for (key, value) in input_values {
                    filled_content =
                        filled_content.replace(&format!("{{{}}}", key), value.to_string().as_str());
                }
                MessageInput {
                    role: message.role.clone(),
                    content: filled_content,
                }
            })
            .collect()
    }

    fn get_tools(
        &self,
        tool_names: Vec<String>,
        custom_templates: Option<Vec<CustomToolTemplate>>,
    ) -> Result<Vec<Arc<dyn Tool>>, ToolError> {
        let mut tools: Vec<Arc<dyn Tool>> = vec![];

        if tool_names.len() == 1 && tool_names[0] == *"ALL".to_string() {
            // Check if serper API is set
            // ALL results in [jina, serper, stock] or [jina, duckduckgo, stock]
            let serper_key = std::env::var("SERPER_API_KEY");
            if serper_key.is_err() {
                tools.push(Arc::new(DDGSearcher::new()));
            } else {
                tools.push(Arc::new(SearchTool {}));
            }
            tools.push(Arc::new(StockScraper::new()));
            tools.push(Arc::new(Jina {}));
        } else {
            if !in_tools(&tool_names) {
                return Err(ToolError::ToolDoesNotExist);
            }

            let _tools: Vec<Arc<dyn Tool>> = tool_names
                .iter()
                .map(|tool| self.get_tool_by_name(tool))
                .collect();

            tools.extend(_tools);
        }

        if let Some(templates) = custom_templates {
            for template in templates {
                let custom_tool = Arc::new(CustomTool::new_from_template(template));
                tools.push(custom_tool);
            }
        }

        Ok(tools)
    }

    fn get_tool_by_name(&self, tool: &str) -> Arc<dyn Tool> {
        match tool {
            "jina" => Arc::new(Jina {}),
            "serper" => Arc::new(SearchTool {}),
            "browserless" => Arc::new(Browserless {}),
            "duckduckgo" => Arc::new(DDGSearcher::new()),
            "stock" => Arc::new(StockScraper::new()),
            "scraper" => Arc::new(Scraper {}),
            _ => Arc::new(Scraper {}),
        }
    }

    async fn handle_output(&self, task: &Task, result: Entry, memory: &mut ProgramMemory) {
        for output in &task.outputs {
            let mut data = result.clone();
            if output.value != R_OUTPUT {
                data = Entry::try_value_or_str(&output.value);
            }
            match output.output_type {
                OutputType::Write => memory.write(output.key.clone(), data.clone()),
                OutputType::Insert => memory.insert(&data).await,
                OutputType::Push => memory.push(output.key.clone(), data.clone()),
            }
        }
    }

    async fn handle_input(
        &self,
        input_value: &InputValue,
        memory: &mut ProgramMemory,
    ) -> MemoryReturnType {
        match input_value.value_type {
            InputValueType::Input => MemoryReturnType::Entry(memory.read(&R_INPUT.to_string())),
            InputValueType::Read => MemoryReturnType::Entry(memory.read(&input_value.key)),
            InputValueType::Peek => MemoryReturnType::Entry(
                memory.peek(&input_value.key, input_value.index.unwrap_or(0)),
            ),
            InputValueType::GetAll => MemoryReturnType::EntryVec(memory.get_all(&input_value.key)),
            InputValueType::Pop => MemoryReturnType::Entry(memory.pop(&input_value.key)),
            InputValueType::String => {
                MemoryReturnType::Entry(Some(Entry::try_value_or_str(&input_value.key)))
            }
            InputValueType::Size => MemoryReturnType::Entry(Some(Entry::try_value_or_str(
                &memory.size(&input_value.key).to_string(),
            ))),
        }
    }

    async fn generate_text(
        &self,
        input: Vec<MessageInput>,
        schema: &Option<String>,
        config: &Config,
    ) -> Result<String, OllamaError> {
        //let json= ChatMessage::assistant(format!("{regex}"));

        let mut messages: Vec<ChatMessage> = input
            .iter()
            .map(|msg| {
                match msg.role.as_str() {
                    "user" => ChatMessage::user(msg.content.clone()),
                    "assistant" => ChatMessage::assistant(msg.content.clone()),
                    _ => ChatMessage::user(msg.content.clone()), // fallback to user
                }
            })
            .collect();

        let response = match self.model.clone().into() {
            ModelProvider::Ollama => {
                return match self.model {
                    Model::Llama3_1_8BTextQ4KM
                    | Model::Llama3_1_8BTextQ8
                    | Model::Llama3_1_70BTextQ4KM
                    | Model::Llama3_2_1BTextQ4KM => {
                        let prompt = input
                            .last()
                            .map(|msg| msg.content.as_str())
                            .unwrap_or_default();
                        let mut msg =
                            GenerationRequest::new(self.model.to_string(), prompt.to_string());
                        let mut ops = GenerationOptions::default();
                        ops = ops.num_predict(config.max_tokens.unwrap_or(250));
                        msg = msg.options(ops);

                        let result = self.llm.generate(msg).await?;

                        Ok(result.response)
                    }
                    _ => {
                        let mut msg = if let Some(schema) = schema {
                            let decoded_schema = match BASE64_STANDARD.decode(schema.as_bytes()) {
                                Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
                                Err(e) => {
                                    warn!("Failed to decode base64 schema: {}", e);
                                    return Err(OllamaError::from(
                                        "Schema format invalid".to_string(),
                                    ));
                                }
                            };
                            messages.insert(0, ChatMessage::assistant(decoded_schema.to_string()));
                            ChatMessageRequest::new(self.model.to_string(), messages)
                                .format(FormatType::Json)
                        } else {
                            ChatMessageRequest::new(self.model.to_string(), messages)
                        };

                        let mut ops = GenerationOptions::default();
                        ops = ops.num_predict(config.max_tokens.unwrap_or(250));
                        msg = msg.options(ops);

                        let result = self.llm.send_chat_messages(msg).await?;

                        Ok(result.message.unwrap().content)
                    }
                }
            }
            ModelProvider::OpenAI => {
                let api_key = std::env::var("OPENAI_API_KEY").expect("$OPENAI_API_KEY is not set");

                let openai_executor = OpenAIExecutor::new(self.model.to_string(), api_key.clone());
                openai_executor.generate_text(input, schema).await?
            }
            ModelProvider::Gemini => {
                let api_key = std::env::var("GEMINI_API_KEY").expect("$GEMINI_API_KEY is not set");
                let max_tokens = config.max_tokens.unwrap_or(800);
                let executor = GeminiExecutor::new(self.model.to_string(), api_key, max_tokens);
                executor.generate_text(input, schema).await?
            }
            ModelProvider::OpenRouter => {
                let api_key =
                    std::env::var("OPENROUTER_API_KEY").expect("$OPENROUTER_API_KEY is not set");

                let openai_executor =
                    OpenRouterExecutor::new(self.model.to_string(), api_key.clone());
                openai_executor.generate_text(input, schema).await?
            }
        };

        Ok(response)
    }

    async fn function_call(
        &self,
        input: Vec<MessageInput>,
        config: &Config,
        raw_mode: bool,
    ) -> Result<String, OllamaError> {
        let oai_parser = Arc::new(OpenAIFunctionCall {});
        let llama_parser = Arc::new(LlamaFunctionCall {});
        let tools = self
            .get_tools(config.tools.clone(), config.custom_tools.clone())
            .unwrap();

        let prompt = input
            .last()
            .map(|msg| msg.content.as_str())
            .unwrap_or_default();

        let result = match self.model.clone().into() {
            ModelProvider::Ollama => {
                //if raw mode is enabled, return only the calls
                let mut request = FunctionCallRequest::new(
                    self.model.to_string(),
                    tools.clone(),
                    vec![ChatMessage::user(prompt.to_string())],
                );

                if raw_mode {
                    request = request.raw_mode();
                }

                let res = self
                    .llm
                    .send_function_call(
                        request,
                        match self.model {
                            Model::NousTheta => llama_parser.clone(),
                            Model::Llama3_1_8B
                            | Model::Llama3_1_8Bf16
                            | Model::Llama3_1_8Bq8
                            | Model::Llama3_2_3B
                            | Model::Llama3_1_70Bq8
                            | Model::Llama3_1_70B => llama_parser.clone(),
                            _ => oai_parser.clone(),
                        },
                    )
                    .await?;
                res.message.unwrap().content
            }
            ModelProvider::OpenAI => {
                let api_key = std::env::var("OPENAI_API_KEY").expect("$OPENAI_API_KEY is not set");

                let openai_executor = OpenAIExecutor::new(self.model.to_string(), api_key.clone());
                openai_executor
                    .function_call(prompt, tools, raw_mode, oai_parser)
                    .await?
            }
            ModelProvider::Gemini => {
                let api_key = std::env::var("GEMINI_API_KEY").expect("$GEMINI_API_KEY is not set");
                let max_tokens = config.max_tokens.unwrap_or(800);
                match self.model{
                    Model::Gemini15Flash | Model::Gemini15Pro => {
                        let executor = GeminiExecutor::new(self.model.to_string(), api_key, max_tokens);
                        executor
                            .function_call(prompt, tools, raw_mode, oai_parser)
                            .await?
                    }
                    _ => return Err(OllamaError::from(format!("Gemini doesn't support function calling for {}. Try using either: Gemini15Flash or Gemini15Pro", self.model)))
                }
            }
            ModelProvider::OpenRouter => {
                let api_key =
                    std::env::var("OPENROUTER_API_KEY").expect("$OPENROUTER_API_KEY is not set");

                let openai_executor =
                    OpenRouterExecutor::new(self.model.to_string(), api_key.clone());
                openai_executor
                    .function_call(prompt, tools, raw_mode, oai_parser)
                    .await?
            }
        };

        Ok(result)
    }

    /// Lists existing models compatible with the `Model` enum.
    ///
    /// Will ignore models that are not compatible with the `Model` enum.
    pub async fn list_local_models(&self) -> Result<Vec<Model>, OllamaError> {
        let local_models = self.llm.list_local_models().await?;

        let local_models = local_models
            .iter()
            .filter_map(|model| Model::try_from(model.clone()).ok())
            .collect();

        Ok(local_models)
    }

    /// Pulls a model if it does not exist locally, only relevant for Ollama models.
    pub async fn pull_model(&self) -> Result<(), OllamaError> {
        if ModelProvider::from(self.model.clone()) == ModelProvider::Ollama {
            let local_models = self.list_local_models().await?;
            if !local_models.contains(&self.model) {
                info!("Pulling model {}, this may take a while.", self.model);
                self.llm
                    .pull_model(self.model.clone().into(), false)
                    .await?;
            } else {
                debug!("Model {} already exists locally", self.model);
            }
        }

        Ok(())
    }

    //randomly sample list of entries
    fn sample(&self, entries: &[Entry]) -> Entry {
        let index = rand::thread_rng().gen_range(0..entries.len());
        entries[index].clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "run this manually"]
    async fn test_pull() {
        let executor = Executor::new(Model::Phi3_5Mini);
        let locals = executor
            .list_local_models()
            .await
            .expect("should list models");
        println!("{:?}", locals);

        executor.pull_model().await.expect("should pull model");
    }
}
