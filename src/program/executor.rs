use super::atomics::{InputValueType, Model, Operator, OutputType, Task, R_INPUT, R_OUTPUT, R_END};
use super::workflow::Workflow;
use crate::memory::types::Entry;
use crate::memory::{MemoryReturnType, ProgramMemory};
use crate::tools::{Browserless, Jina, SearchTool};

use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use log::{info, warn, error, debug};
use rand::seq::SliceRandom;
use colored::*;

use ollama_rs::{
    error::OllamaError,
    generation::chat::request::ChatMessageRequest,
    generation::chat::ChatMessage,
    generation::functions::tools::StockScraper,
    generation::functions::{FunctionCallRequest, NousFunctionCall, OpenAIFunctionCall},
    generation::options::GenerationOptions,
    Ollama,
};

fn log_colored(msg: &str) {

    let colors = vec![
        "red", "green", "yellow", "blue", "magenta", "cyan",
    ];

    let color = colors.choose(&mut rand::thread_rng()).unwrap();
    let colored_msg = match *color {
        "red" => msg.red(),
        "green" => msg.green(),
        "yellow" => msg.yellow(),
        "blue" => msg.blue(),
        "magenta" => msg.magenta(),
        "cyan" => msg.cyan(),
        "white" => msg.white(),
        _ => msg.normal(), // default color if none matched
    };
    warn!("{}", colored_msg);
}


pub struct Executor {
    model: Model,
    llm: Ollama,
}

impl Executor {
    pub fn new(model: Model) -> Self {
        let llm = Ollama::default();
        Executor { model, llm }
    }

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

                if &edge.source == R_END {
                    warn!("Successfully completed the workflow");
                    break
                }

                if let Some(task) = workflow.get_tasks_by_id(&edge.source) {
                    let is_done = self.execute_task(task, memory.borrow_mut()).await;

                    current_step = if is_done {
                        warn!("[{}] completed successfully, stepping into [{}]", &edge.source, &edge.target);
                        workflow.get_step_by_id(&edge.target)
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

    async fn execute_task(&self, task: &Task, memory: &mut ProgramMemory) -> bool {
        info!("Executing task: {} with id {}", &task.name, &task.id);
        info!("Using operator: {:?}", &task.operator);

        let mut input_map: HashMap<String, String> = HashMap::new();
        for input in &task.inputs {
            let value: MemoryReturnType = match input.value.value_type {
                InputValueType::Input => {
                    MemoryReturnType::EntryRef(memory.read(&R_INPUT.to_string()))
                }
                InputValueType::Read => MemoryReturnType::EntryRef(memory.read(&input.value.key)),
                InputValueType::Peek => MemoryReturnType::EntryRef(
                    memory.peek(&input.value.key, input.value.index.unwrap_or(0)),
                ),
                InputValueType::GetAll => {
                    MemoryReturnType::EntryVec(memory.get_all(&input.value.key))
                }
                InputValueType::Pop => MemoryReturnType::Entry(memory.pop(&input.value.key)),
                InputValueType::Search => MemoryReturnType::EntryVec(
                    memory.search(&Entry::from_str(&input.value.key)).await,
                ),
                InputValueType::String => {
                    MemoryReturnType::Entry(Some(Entry::from_str(&input.value.key)))
                }
            };

            if input.required && value.is_none() {
                return false;
            }
            input_map.insert(input.name.clone(), value.to_string());
        }

        match task.operator {
            Operator::Generation => {
                let prompt = self.fill_prompt(&task.prompt, &input_map);
                let result = self.generate_text(&prompt).await;
                if result.is_err() {
                    error!("Error generating text: {:?}", result.err().unwrap());
                    return false;
                }
                debug!("Prompt: {}", &prompt);
                log_colored(format!("Operator: {:?}. Output: {:?}", &task.operator ,&result).as_str());
                let result_entry = Entry::from_str(&result.unwrap());
                self.handle_output(task, result_entry, memory).await;
            }
            Operator::FunctionCalling => {
                let prompt = self.fill_prompt(&task.prompt, &input_map);
                let result = self.function_call(&prompt).await;
                if result.is_err() {
                    error!("Error generating text: {:?}", result.err().unwrap());
                    return false;
                }
                debug!("Prompt: {}", &prompt);
                log_colored(format!("Operator: {:?}. Output: {:?}", &task.operator ,&result).as_str());
                let result_entry = Entry::from_str(&result.unwrap());
                self.handle_output(task, result_entry, memory).await;
            }
            Operator::Check => {
                let input = self.prepare_check(input_map);
                let result = self.check(&input.0, &input.1);
                return result;
            }
            Operator::FuzzyCheck => {
                let input = self.prepare_check(input_map);
                let result = self.fuzzy_check(&input.0, &input.1);
                return result;
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
        let input = &input_map.get("input");
        let expected = &input_map.get("expected");

        if let Some(i) = input {
            if let Some(e) = expected {
                return (i.to_string().clone(), e.to_string().clone());
            }
        }
        ("+".to_string(), "-".to_string())
    }

    async fn handle_output(&self, task: &Task, result: Entry, memory: &mut ProgramMemory) {
        for output in &task.outputs {
            let mut data = result.clone();
            if &output.value != R_OUTPUT {
                data = Entry::from_str(&output.value);
            }
            match output.output_type {
                OutputType::Write => memory.write(output.key.clone(), data.clone()),
                OutputType::Insert => memory.insert(&data).await,
                OutputType::Push => memory.push(output.key.clone(), data.clone()),
            }
        }
    }

    async fn generate_text(&self, prompt: &str) -> Result<String, OllamaError> {
        let user_message = ChatMessage::user(prompt.to_string());

        let mut ops = GenerationOptions::default();
        ops = ops.num_predict(150);
        ops = ops.num_ctx(4096);

        let mut msg = ChatMessageRequest::new(self.model.to_string(), vec![user_message]);
        msg = msg.options(ops);

        let result = self.llm.send_chat_messages(msg).await?;

        Ok(result.message.unwrap().content)
    }

    async fn function_call(&self, prompt: &str) -> Result<String, OllamaError> {
        //allow to switch tools here
        let oai_parser = Arc::new(OpenAIFunctionCall {});
        let nous_parser = Arc::new(NousFunctionCall {});
        let scraper_tool = Arc::new(Browserless {});
        let search_tool = Arc::new(SearchTool {});
        let jina_tool = Arc::new(Jina {});
        let stock_scraper = Arc::new(StockScraper::new());

        let result = match self.model {
            Model::NousTheta => {
                self.llm
                    .send_function_call(
                        FunctionCallRequest::new(
                            self.model.to_string(),
                            vec![
                                stock_scraper.clone(),
                                search_tool.clone(),
                                jina_tool.clone(),
                            ],
                            vec![ChatMessage::user(prompt.to_string())],
                        ),
                        nous_parser.clone(),
                    )
                    .await
            }
            _ => {
                self.llm
                    .send_function_call(
                        FunctionCallRequest::new(
                            self.model.to_string(),
                            vec![jina_tool.clone(), search_tool.clone()],
                            vec![ChatMessage::user(prompt.to_string())],
                        ),
                        oai_parser.clone(),
                    )
                    .await
            }
        }?;

        Ok(result.message.unwrap().content)
    }

    fn check(&self, input: &str, expected: &str) -> bool {
        input == expected
    }

    fn fuzzy_check(&self, input: &str, expected: &str) -> bool {
        input == expected
    }
}
