use super::atomics::*;
use super::workflow::Workflow;
use crate::memory::types::Entry;
use crate::memory::{MemoryReturnType, ProgramMemory};
use crate::program::errors::ToolError;
use crate::tools::{Browserless, CustomTool, Jina, SearchTool};

use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use colored::*;
use langchain_rust::language_models::llm::LLM;
use langchain_rust::llm::OpenAI;
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
    /// Create a new Executor
    pub fn new(model: Model) -> Self {
        let llm = Ollama::default();
        Executor { model, llm }
    }

    /// Executes the workflow
    pub async fn execute(
        &self,
        input: Option<&Entry>,
        workflow: Workflow,
        memory: &mut ProgramMemory,
    ) {
        let config = workflow.get_config();
        let max_steps = config.max_steps;
        let max_time = config.max_time;

        warn!("------------------");
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
                            let eval = condition
                                .expression
                                .evaluate(&value.to_string(), &condition.expected);
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
    }

    async fn execute_task(&self, task: &Task, memory: &mut ProgramMemory, config: &Config) -> bool {
        info!("Executing task: {} with id {}", &task.name, &task.id);
        info!("Using operator: {:?}", &task.operator);

        let mut input_map: HashMap<String, String> = HashMap::new();
        for input in &task.inputs {
            let value = self.handle_input(&input.value, memory).await;
            if input.required && value.is_none() {
                return false;
            }
            input_map.insert(input.name.clone(), value.to_string());
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
                let input = self.prepare_check(input_map);
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
            Operator::End => {}
        };

        true
    }

    fn fill_prompt(&self, prompt: &str, input_values: &HashMap<String, String>) -> String {
        let mut filled_prompt = prompt.to_string();
        for (key, value) in input_values {
            filled_prompt = filled_prompt.replace(&format!("{{{}}}", key), value);
        }
        filled_prompt
    }

    fn prepare_check(&self, input_map: HashMap<String, String>) -> (String, String) {
        let input = &input_map.get(R_OUTPUTS);
        let expected = &input_map.get(R_EXPECTED);

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
        if !in_tools(&tool_names) {
            return Err(ToolError::ToolDoesNotExist);
        }
        let mut tools: Vec<Arc<dyn Tool>> = tool_names
            .iter()
            .map(|tool| self.get_tool_by_name(tool))
            .collect();

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

    async fn handle_input<'a>(
        &'a self,
        input_value: &'a InputValue,
        memory: &'a mut ProgramMemory,
    ) -> MemoryReturnType<'a> {
        return match input_value.value_type {
            InputValueType::Input => MemoryReturnType::EntryRef(memory.read(&R_INPUT.to_string())),
            InputValueType::Read => MemoryReturnType::EntryRef(memory.read(&input_value.key)),
            InputValueType::Peek => MemoryReturnType::EntryRef(
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
        };
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
                let llm = OpenAI::default().with_model(self.model.to_string());

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
                self.llm
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
                    .await
            }
            ModelProvider::OpenAI => {
                unimplemented!("OpenAI function calling is not implemented in this build");
            }
        }?;

        Ok(result.message.unwrap().content)
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
