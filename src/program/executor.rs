use super::workflow::Workflow;
use super::atomics::{Task, InputValueType, Operator, OutputType};
use crate::memory::types::Entry;
use crate::memory::{ProgramMemory, MemoryReturnType};
use parking_lot::Mutex;
use std::sync::Arc;
use std::collections::HashMap;
use std::time::Instant;


pub struct Executor {
}

impl Executor {
    pub fn new() -> Self {
        Executor {
        }
    }

    pub async fn execute(&self, workflow: Workflow, memory: Arc<Mutex<ProgramMemory>>) {
        let config = workflow.get_config();
        let max_steps = config.max_steps;
        let max_time = config.max_time;

        let mut current_step = 0;
        let start = Instant::now();



        while current_step < max_steps && start.elapsed().as_secs() < max_time {
            if let Some(edge) = workflow.get_step(current_step) {
                let task = workflow.get_tasks_by_id(&edge.source);

                if let Some(task) = task {
                    if !self.execute_task(task, memory.clone()).await {
                        if let Some(fallback_task_id) = &edge.fallback {
                            let fallback_task = workflow.get_tasks_by_id(fallback_task_id);
                            if let Some(fallback_task) = fallback_task {
                                self.execute_task(fallback_task, memory.clone()).await;
                            }
                        }
                    }
                }
    
                current_step += 1;
            }
            else{
                break;
            }
        }
    }

    async fn execute_task(&self, task: &Task, memory: Arc<Mutex<ProgramMemory>>) -> bool {

        let mut input_map: HashMap<String, String> = HashMap::new();
        for input in &task.inputs {
            let mut memory_lock = memory.lock();
            let value: MemoryReturnType = match input.value.value_type {
                InputValueType::Read => MemoryReturnType::EntryRef(memory_lock.read(&input.value.key)),
                InputValueType::Peek => MemoryReturnType::EntryRef(memory_lock.peek(&input.value.key, input.value.index.unwrap_or(0))),
                InputValueType::Pop => MemoryReturnType::Entry(memory_lock.pop(&input.value.key)),
                InputValueType::Search => MemoryReturnType::EntryVec(memory_lock.search(&Entry::from_str(&input.value.key)).await),
                InputValueType::String => MemoryReturnType::Entry(Some(Entry::from_str(&input.value.key))),
            };

            if input.required && value.is_none() {
                return false;
            }
            input_map.insert(input.name.clone(), value.to_string());
        }

        match task.operator {
            Operator::Generation => {
                let prompt = self.fill_prompt(&task.prompt, &input_map);
                let result = self.generate_text(&prompt);
                let result_entry = Entry::from_str(&result.unwrap());
                self.handle_output(task, result_entry, memory).await;
            },
            Operator::FunctionCalling => {
                let prompt = self.fill_prompt(&task.prompt, &input_map);
                let result = self.function_call(&prompt);
                let result_entry = Entry::from_str(&result.unwrap());
                self.handle_output(task, result_entry, memory).await;
            }, 
            Operator::Check => {
                let input = self.prepare_check(input_map);
                let result = self.check(&input.0, &input.1);
                return result;
            }, 
            Operator::FuzzyCheck => {
                let input = self.prepare_check(input_map);
                let result = self.fuzzy_check(&input.0, &input.1);
                return result;
            },
            Operator::End => {},
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

    //TODO: "result" keyword is reserved, not sure If any other output type is needed, may remove value from Ouput enum
    async fn handle_output(&self, task: &Task, result: Entry, memory: Arc<Mutex<ProgramMemory>>) {
        for output in &task.outputs {
            let mut memory_lock = memory.lock();
            match output.output_type {
                OutputType::Write => memory_lock.write(output.key.clone(), result.clone()),
                OutputType::Insert => memory_lock.insert(&result).await,
                OutputType::Push => memory_lock.push(output.key.clone(), result.clone()),
            }
        }
    }

    fn generate_text(&self, prompt: &str) -> Option<String> {
        // Placeholder implementation
        None
    }

    fn function_call(&self, prompt: &str) -> Option<String> {
        // Placeholder implementation
        None
    }

    fn check(&self, input: &str, expected: &str) -> bool {
        input == expected
    }

    fn fuzzy_check(&self, input: &str, expected: &str) -> bool {
        input == expected
    }
}