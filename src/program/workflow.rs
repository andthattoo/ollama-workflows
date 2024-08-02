use super::atomics::{Config, Edge, Task, TaskOutput};
use crate::memory::types::{Entry, StackPage, ID};
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::collections::HashMap;
use super::parser::{parse_step,lexer_tasks,parse_tasks};

fn split_json_string(s: &str) -> Vec<String> {
    let substrings: Vec<&str> = s.split("},{").collect();
    let mut result: Vec<String> = Vec::new();

    for (index, substring) in substrings.iter().enumerate() {
        match index {
            0 => result.push(format!("{}{}", substring, "}")), // Add } to the first substring
            _ if index == substrings.len() - 1 => result.push(format!("{{{}", substring)), // Add { to the last substring
            _ => result.push(format!("{{{}{}", substring, "}")), // Add both { and } to middle substrings
        }
    }

    result
}
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

fn deserialize_tasks<'de, D>(
    deserializer: D,
) -> Result<Vec<Task>, D::Error>
where
    D: Deserializer<'de>,
{
    
    let value: Option<Value> = Option::deserialize(deserializer)?;
    let mut string_representation = String::new();
        // Convert the entire deserialized object into a string representation
    string_representation = serde_json::to_string(&value).unwrap();
   // error handling
    let trimmed_string = string_representation
    .trim_start_matches('[')
    .trim_end_matches(']');
    let split_strings = split_json_string(trimmed_string);
    let mut tasks = Vec::<Task>::new();
    for lines in split_strings {
       let tokens = lexer_tasks(&lines);
       let task = parse_tasks(tokens);
       tasks.push(task)
    }

   
    Ok(tasks)
}

fn deserialize_steps<'de, D>(
    deserializer: D,
) -> Result<Vec<Edge>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<Value> = Option::deserialize(deserializer)?;
    // Assuming value contains a single string with multiple lines
    // it will work if its single line as well.
    let multi_line_string = value.as_ref().and_then(|v| v.as_str()).unwrap_or("");

    // Split the string by newlines to get individual lines.
    // last line is empty, so we filter it out.
    let predicate = |line: &&str| !line.trim().is_empty();
    let lines: Vec<&str> = multi_line_string.split('.').filter(predicate).collect();

    let mut steps = Vec::<Edge>::new();
      

    for line in lines {
        if !line.trim().is_empty() {
            let trimmed_string = line.trim_start();
            let (_, step) = parse_step(&trimmed_string).expect("Failed to parse step");
            steps.push(step);
        }
    }
        
    Ok(steps)
   
    // } else {
    //     Ok(None) 
    // maybe not required as we are returning empty vec
    // }
}

/// Workflow serves as a container for the tasks and steps that make up a workflow.
#[derive(Debug, serde::Deserialize)]
pub struct Workflow {
    config: Config,
    #[serde(default, deserialize_with = "deserialize_external_memory")]
    pub external_memory: Option<HashMap<ID, StackPage>>,
    #[serde(deserialize_with = "deserialize_tasks")]
    tasks: Vec<Task>,
    #[serde(default, deserialize_with = "deserialize_steps")]
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

    /// Creates a new Workflow from a JSON file.
    pub fn new_from_json(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = std::fs::File::open(path)?;
        let mut reader = std::io::BufReader::new(file);
        let serde_object = serde_json5::from_reader(&mut reader)?;
        let workflow: Workflow = serde_object;
        Ok(workflow)
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
