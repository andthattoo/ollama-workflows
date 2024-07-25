use super::atomics::{Config, Edge, Task, TaskOutput};
use crate::memory::types::{Entry, StackPage, ID};
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::collections::HashMap;

/// Custom deserializer for external memory.
fn deserialize_external_memory<'de, D>(
    deserializer: D,
) -> Result<Option<HashMap<ID, StackPage>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<Value> = Option::deserialize(deserializer)?;

    if let Some(value) = value {
        let map = value
            .as_object()
            .ok_or_else(|| serde::de::Error::custom("Expected a map"))?;

        let mut external_memory = HashMap::new();

        for (key, val) in map {
            if let Some(array) = val.as_array() {
                let mut stack_page = Vec::new();
                for item in array {
                    if let Some(s) = item.as_str() {
                        stack_page.push(Entry::String(s.to_string()));
                    } else if item.is_object() {
                        stack_page.push(Entry::Json(item.clone()));
                    } else {
                        return Err(serde::de::Error::custom("Invalid entry format"));
                    }
                }
                external_memory.insert(key.clone(), stack_page);
            }
        }

        Ok(Some(external_memory))
    } else {
        Ok(None)
    }
}

/// Workflow serves as a container for the tasks and steps that make up a workflow.
#[derive(Debug, serde::Deserialize)]
pub struct Workflow {
    config: Config,
    #[serde(default, deserialize_with = "deserialize_external_memory")]
    pub external_memory: Option<HashMap<ID, StackPage>>,
    tasks: Vec<Task>,
    steps: Vec<Edge>,
    return_value: TaskOutput,
}

impl Workflow {
    pub fn new(
        tasks: Vec<Task>,
        steps: Vec<Edge>,
        config: Config,
        external_memory: Option<HashMap<ID, StackPage>>,
        return_value: TaskOutput,
    ) -> Self {
        Workflow {
            config,
            external_memory,
            tasks,
            steps,
            return_value,
        }
    }

    fn new_from_json5(reader: std::io::BufReader<std::fs::File>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut workflow5 = String::new();
        reader.read_to_string(&mut workflow5)?;
        let workflow5: Value = json5::from_str(&workflow5)?;
        let config = workflow5.get("config").unwrap();
        let tasks = workflow5.get("tasks").unwrap();
        let steps = workflow5.get("steps").unwrap();
        let return_value = workflow5.get("return_value").unwrap();
        let external_memory = workflow5.get("external_memory").unwrap();
        let config: Config = serde_json::from_value(config.clone())?;
        let tasks: Vec<Task> = serde_json::from_value(tasks.clone())?;
        let steps: Vec<Edge> = serde_json::from_value(steps.clone())?;
        let return_value: TaskOutput = serde_json::from_value(return_value.clone())?;
        let external_memory: Option<HashMap<ID, StackPage>> = serde_json::from_value(external_memory.clone())?;
        Ok(Workflow::new(tasks, steps, config, external_memory, return_value))
    }
    /// Creates a new Workflow from a JSON file.
    pub fn new_from_json(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        // json5 style
        let workflow5  = Self::new_from_json5(reader)?;
        //let workflow: Workflow = serde_json::from_reader(reader)?;
        Ok(workflow5)
    }
}

impl Workflow {
    /// Returns a reference to the configuration of the workflow.
    pub fn get_config(&self) -> &Config {
        &self.config
    }
    /// Returns a reference to the tasks of the workflow.
    pub fn get_tasks(&self) -> &Vec<Task> {
        &self.tasks
    }
    /// Returns a reference to the steps of the workflow.
    pub fn get_workflow(&self) -> &Vec<Edge> {
        &self.steps
    }
    /// Returns a reference to the return value of the workflow.
    pub fn get_return_value(&self) -> &TaskOutput {
        &self.return_value
    }
    /// Returns a reference to the task at the specified index.
    pub fn get_step(&self, index: u32) -> Option<&Edge> {
        self.steps.get(index as usize)
    }
    /// Returns a reference to the step for specified task_id.
    pub fn get_step_by_id(&self, task_id: &str) -> Option<&Edge> {
        self.steps.iter().find(|edge| edge.source == task_id)
    }
    /// Returns a reference to the task at the specified task_id.
    pub fn get_tasks_by_id(&self, task_id: &str) -> Option<&Task> {
        self.tasks.iter().find(|task| task.id == task_id)
    }
}
