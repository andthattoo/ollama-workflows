use super::workflow::Workflow;
use super::atomics::{Task, InputValueType, Operator, OutputType};
use crate::memory::ProgramMemory;
use parking_lot::Mutex;
use std::sync::Arc;
use std::collections::HashMap;

struct Executor {
}

impl Executor {
    fn new() -> Self {
        Executor {
        }
    }

    fn execute(&self, workflow: Workflow, memory: Arc<ProgramMemory>) {
        let max_steps = workflow.get_config().max_steps;
        let max_time = workflow.get_config().max_time;

        let mut current_step = 0;
        let mut elapsed_time = 0;

        while current_step < max_steps && elapsed_time < max_time {
            if let Some(edge) = workflow.get_step(current_step) {
                let task = self.find_task(&edge.source);

                if let Some(task) = task {
                    if !self.execute_task(task) {
                        if let Some(fallback_task_id) = &edge.fallback {
                            let fallback_task = self.find_task(fallback_task_id);
                            if let Some(fallback_task) = fallback_task {
                                self.execute_task(fallback_task);
                            }
                        }
                    }
                }
    
                current_step += 1;
                elapsed_time += 1; // Placeholder for time tracking
            }
            else{
                break;
            }
        }
    }

    fn find_task(&self, task_id: &str) -> Option<&Task> {
        self.workflow.get_tasks_by_id(task_id)
    }

    fn execute_task(&mut self, task: &Task) -> bool {
        let mut input_values = HashMap::new();

        for input in &task.inputs {
            let value = match input.value.value_type {
                InputValueType::Read => self.read(&input.value.key),
                // Handle other input value types as needed
            };

            if input.required && value.is_none() {
                // Handle missing required input
                return false;
            }

            if let Some(value) = value {
                input_values.insert(input.name.clone(), value);
            }
        }

        let result = match task.operator {
            Operator::Generation => self.generate_text(&task.prompt, &input_values),
            // Handle other operators as needed
        };

        if let Some(result) = result {
            for output in &task.outputs {
                match output.output_type {
                    OutputType::Write => self.write(&output.key, &result),
                    // Handle other output types as needed
                }
            }
            true
        } else {
            false
        }
    }

    fn generate_text(&self, prompt: &str, input_values: &HashMap<String, String>) -> Option<String> {
        // Placeholder implementation
        let mut generated_text = prompt.to_string();
        for (key, value) in input_values {
            generated_text = generated_text.replace(&format!("{{{}}}", key), value);
        }
        Some(generated_text)
    }

    fn write(&mut self, key: &str, value: &str) {
        self.cache.insert(key.to_string(), value.to_string());
    }

    fn read(&self, key: &str) -> Option<String> {
        self.cache.get(key).cloned()
    }
}