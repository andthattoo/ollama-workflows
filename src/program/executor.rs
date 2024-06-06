use super::atomics::{InputValueType, Model, Operator, OutputType, Task};
use super::workflow::Workflow;
use crate::memory::types::Entry;
use crate::memory::{MemoryReturnType, ProgramMemory};
use crate::tools::{Browserless, SearchTool};

use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use ollama_rs::{
    error::OllamaError,
    generation::chat::request::ChatMessageRequest,
    generation::chat::ChatMessage,
    generation::functions::tools::StockScraper,
    generation::functions::{FunctionCallRequest, NousFunctionCall, OpenAIFunctionCall},
    generation::options::GenerationOptions,
    Ollama,
};

pub struct Executor {
    model: Model,
    llm: Ollama,
}

impl Executor {
    pub fn new(model: Model) -> Self {
        let llm = Ollama::default();
        Executor { model, llm }
    }

    pub async fn execute(&self, workflow: Workflow, memory: &mut ProgramMemory) {
        let config = workflow.get_config();
        let max_steps = config.max_steps;
        let max_time = config.max_time;

        let mut current_step = 0;
        let start = Instant::now();

        while current_step < max_steps && start.elapsed().as_secs() < max_time {
            if let Some(edge) = workflow.get_step(current_step) {
                let task = workflow.get_tasks_by_id(&edge.source);

                if let Some(task) = task {
                    if !self.execute_task(task, memory.borrow_mut()).await {
                        if let Some(fallback_task_id) = &edge.fallback {
                            let fallback_task = workflow.get_tasks_by_id(fallback_task_id);
                            if let Some(fallback_task) = fallback_task {
                                self.execute_task(fallback_task, memory.borrow_mut()).await;
                            }
                        }
                    }
                }

                current_step += 1;
            } else {
                break;
            }
        }
    }

    async fn execute_task(&self, task: &Task, memory: &mut ProgramMemory) -> bool {
        let mut input_map: HashMap<String, String> = HashMap::new();
        for input in &task.inputs {
            //let mut memory_lock = memory.lock();
            let value: MemoryReturnType = match input.value.value_type {
                InputValueType::Read => MemoryReturnType::EntryRef(memory.read(&input.value.key)),
                InputValueType::Peek => MemoryReturnType::EntryRef(
                    memory.peek(&input.value.key, input.value.index.unwrap_or(0)),
                ),
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
                    return false;
                }
                let result_entry = Entry::from_str(&result.unwrap());
                self.handle_output(task, result_entry, memory).await;
            }
            Operator::FunctionCalling => {
                let prompt = self.fill_prompt(&task.prompt, &input_map);
                let result = self.function_call(&prompt).await;
                if result.is_err() {
                    return false;
                }
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
            match output.output_type {
                OutputType::Write => memory.write(output.key.clone(), result.clone()),
                OutputType::Insert => memory.insert(&result).await,
                OutputType::Push => memory.push(output.key.clone(), result.clone()),
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
        let parser = Arc::new(OpenAIFunctionCall {});
        let scraper_tool = Arc::new(Browserless {});
        let search_tool = Arc::new(SearchTool {});
        let stock_scraper = Arc::new(StockScraper::new());
        let result = self
            .llm
            .send_function_call(
                FunctionCallRequest::new(
                    self.model.to_string(),
                    vec![
                        stock_scraper.clone(),
                        search_tool.clone(),
                        scraper_tool.clone(),
                    ],
                    vec![ChatMessage::user(prompt.to_string())],
                ),
                parser.clone(),
            )
            .await?;

        Ok(result.message.unwrap().content)
    }

    fn check(&self, input: &str, expected: &str) -> bool {
        input == expected
    }

    fn fuzzy_check(&self, input: &str, expected: &str) -> bool {
        input == expected
    }
}
