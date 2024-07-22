use super::atomics::*;
use super::workflow::Workflow;
use crate::memory::types::Entry;
use crate::memory::{MemoryReturnType, ProgramMemory};
use crate::program::errors::ToolError;
use crate::tools::{Browserless, CustomTool, Jina, SearchTool};

use rand::Rng;
use serde_json::Value;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use colored::*;
use langchain_rust::language_models::llm::LLM;
use langchain_rust::llm::OpenAI;

use openai_dive::v1::api::Client;
use openai_dive::v1::resources::chat::*;

use log::{debug, error, info, warn};
use rand::seq::SliceRandom;

use ollama_rs::{
    error::OllamaError,
    generation::chat::request::ChatMessageRequest,
    generation::chat::ChatMessage,
    generation::functions::tools::StockScraper,
    generation::functions::tools::Tool,
    generation::functions::{
        DDGSearcher, FunctionCallRequest, NousFunctionCall, OpenAIFunctionCall, Scraper,
    },
    generation::options::GenerationOptions,
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
        workflow: Workflow,
        memory: &mut ProgramMemory,
    ) -> String {
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
                    let is_done = self.execute_task(task, memory.borrow_mut(), config).await;

                    current_step = if is_done {
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
                        workflow.get_step_by_id(fallback)
                    } else {
                        warn!("{} failed, halting beacause of no fallback", &edge.source);
                        break;
                    };
                } else {
                    warn!("Task with id [{}] not found, halting", &edge.source);
                    break;
                }
            } else {
                break;
            }
            num_steps += 1;
        }
        let rv = workflow.get_return_value();
        let return_value = self.handle_input(&rv.input, memory).await;
        let mut return_string = return_value.to_string().clone();

        if rv.to_json.is_some() && rv.to_json.unwrap() {
            let res = return_value.to_json();
            if let Some(result) = res {
                return result;
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
        return_string
    }

    async fn execute_task(&self, task: &Task, memory: &mut ProgramMemory, config: &Config) -> bool {
        info!("Executing task: {} with id {}", &task.name, &task.id);
        info!("Using operator: {:?}", &task.operator);

        let mut input_map: HashMap<String, MemoryReturnType> = HashMap::new();
        for input in &task.inputs {
            let value = self.handle_input(&input.value, memory).await;
            if input.required && value.is_none() {
                return false;
            }
            input_map.insert(input.name.clone(), value.clone());
        }

        match task.operator {
            Operator::Generation => {
                let prompt = self.fill_prompt(&task.prompt, &input_map);
                let result = self.generate_text(&prompt, config).await;
                if result.is_err() {
                    error!("Error generating text: {:?}", result.err().unwrap());
                    return false;
                }
                debug!("Prompt: {}", &prompt);
                log_colored(
                    format!("Operator: {:?}. Output: {:?}", &task.operator, &result).as_str(),
                );
                let result_entry = Entry::try_value_or_str(&result.unwrap());
                self.handle_output(task, result_entry, memory).await;
            }
            Operator::FunctionCalling => {
                let prompt = self.fill_prompt(&task.prompt, &input_map);
                info!("Prompt: {}", &prompt);
                let result = self.function_call(&prompt, config).await;
                if result.is_err() {
                    error!("Error generating text: {:?}", result.err().unwrap());
                    return false;
                }
                debug!("Prompt: {}", &prompt);
                log_colored(
                    format!("Operator: {:?}. Output: {:?}", &task.operator, &result).as_str(),
                );
                let result_entry = Entry::try_value_or_str(&result.unwrap());
                self.handle_output(task, result_entry, memory).await;
            }
            Operator::Check => {
                let input = self.prepare_check(&input_map);
                let result = self.check(&input.0, &input.1);
                return result;
            }
            Operator::Search => {
                let prompt = self.fill_prompt(&task.prompt, &input_map);
                let result = memory.search(&Entry::try_value_or_str(&prompt)).await;
                if result.is_none() {
                    error!("Error searching: {:?}", "No results found");
                    return false;
                }
                log_colored(
                    format!("Operator: {:?}. Output: {:?}", &task.operator, &result).as_str(),
                );

                let ent_str = MemoryReturnType::EntryVec(result).to_string();
                let result_entry = Entry::try_value_or_str(&ent_str);
                self.handle_output(task, result_entry, memory).await;
            }
            Operator::Sample => {
                // Read Stack for each key in the inputs
                // Sample from the stack
                // fill prompts with values
                // write to memory

                let mut prompt = self.fill_prompt(&task.prompt, &input_map);
                for (key, value) in &input_map {
                    let v = Vec::<Entry>::from(value.clone());
                    if !v.is_empty() {
                        error!("Input for Sample operator cannot be GetAll");
                        return false;
                    } else {
                        let stack_lookup = value.to_string();
                        let entry = memory.get_all(&stack_lookup);
                        if entry.is_none() {
                            error!("Error sampling: {:?}", key);
                            return false;
                        }
                        let sample = self.sample(&entry.unwrap());
                        prompt.push_str(&format!(": {}", sample));
                    }
                }
                info!("Sampled: {}", &prompt);
                self.handle_output(task, Entry::try_value_or_str(&prompt), memory)
                    .await;
            }
            Operator::End => {}
        };

        true
    }

    fn fill_prompt(
        &self,
        prompt: &str,
        input_values: &HashMap<String, MemoryReturnType>,
    ) -> String {
        let mut filled_prompt = prompt.to_string();
        for (key, value) in input_values {
            filled_prompt =
                filled_prompt.replace(&format!("{{{}}}", key), value.to_string().as_str());
        }
        filled_prompt
    }

    fn prepare_check(&self, input_map: &HashMap<String, MemoryReturnType>) -> (String, String) {
        let input = input_map.get(R_OUTPUTS);
        let expected = input_map.get(R_EXPECTED);

        if let Some(i) = input {
            if let Some(e) = expected {
                return (
                    i.to_string()
                        .trim()
                        .replace('\n', "")
                        .to_lowercase()
                        .clone(),
                    e.to_string()
                        .trim()
                        .replace('\n', "")
                        .to_lowercase()
                        .clone(),
                );
            }
        }
        ("+".to_string(), "-".to_string())
    }

    fn get_tools(
        &self,
        tool_names: Vec<String>,
        custom_template: Option<CustomToolTemplate>,
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

        if let Some(template) = custom_template {
            let custom_tool = Arc::new(CustomTool::new_from_template(template));
            tools.push(custom_tool);
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

    async fn generate_text(&self, prompt: &str, config: &Config) -> Result<String, OllamaError> {
        let user_message = ChatMessage::user(prompt.to_string());

        let response = match self.model.clone().into() {
            ModelProvider::Ollama => {
                let mut msg = ChatMessageRequest::new(self.model.to_string(), vec![user_message]);
                let mut ops = GenerationOptions::default();
                ops = ops.num_predict(config.max_tokens.unwrap_or(250));
                msg = msg.options(ops);

                let result = self.llm.send_chat_messages(msg).await?;

                result.message.unwrap().content
            }
            ModelProvider::OpenAI => {
                let llm: OpenAI<langchain_rust::llm::OpenAIConfig> =
                    OpenAI::default().with_model(self.model.to_string());

                llm.invoke(prompt)
                    .await
                    .map_err(|e| OllamaError::from(format!("Could not generate text: {:?}", e)))?
            }
        };

        Ok(response)
    }

    async fn function_call(&self, prompt: &str, config: &Config) -> Result<String, OllamaError> {
        let oai_parser = Arc::new(OpenAIFunctionCall {});
        let nous_parser = Arc::new(NousFunctionCall {});
        let tools = self
            .get_tools(config.tools.clone(), config.custom_tool.clone())
            .unwrap();

        let result = match self.model.clone().into() {
            ModelProvider::Ollama => {
                let res = self
                    .llm
                    .send_function_call(
                        FunctionCallRequest::new(
                            self.model.to_string(),
                            tools,
                            vec![ChatMessage::user(prompt.to_string())],
                        ),
                        match self.model {
                            Model::NousTheta => nous_parser.clone(),
                            _ => oai_parser.clone(),
                        },
                    )
                    .await?;
                res.message.unwrap().content
            }
            ModelProvider::OpenAI => {
                let api_key = std::env::var("OPENAI_API_KEY").expect("$OPENAI_API_KEY is not set");
                let client = Client::new(api_key);

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

                let messages = vec![ChatMessageBuilder::default()
                    .content(ChatMessageContent::Text(prompt.to_string()))
                    .build()
                    .expect("OpenAI function call message build error")];

                let parameters = ChatCompletionParametersBuilder::default()
                    .model(self.model.to_string())
                    .messages(messages)
                    .tools(openai_tools)
                    .build()
                    .expect("Error while building tools.");

                let result = client.chat().create(parameters).await.expect("msg");
                let message = result.choices[0].message.clone();

                let mut results = Vec::<String>::new();
                if let Some(tool_calls) = message.tool_calls {
                    for tool_call in tool_calls {
                        for tool in &tools {
                            if tool.name().to_lowercase().replace(' ', "_")
                                == tool_call.function.name
                            {
                                let tool_params: Value =
                                    serde_json::from_str(&tool_call.function.arguments)?;
                                let res = oai_parser
                                    .function_call_with_history(
                                        tool_call.function.name.clone(),
                                        tool_params,
                                        tool.clone(),
                                    )
                                    .await;
                                if let Ok(result) = res {
                                    results.push(result.message.unwrap().content);
                                } else {
                                    return Err(OllamaError::from(format!(
                                        "Could not generate text: {:?}",
                                        res.err().unwrap()
                                    )));
                                }
                            }
                        }
                    }
                }
                results.join("\n")
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

    #[inline]
    fn check(&self, input: &str, expected: &str) -> bool {
        input == expected
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
        let executor = Executor::new(Model::Phi3Mini);
        let locals = executor
            .list_local_models()
            .await
            .expect("should list models");
        println!("{:?}", locals);

        executor.pull_model().await.expect("should pull model");
    }
}
