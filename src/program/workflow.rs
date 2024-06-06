use super::atomics::{Config, Edge, Task};

#[derive(Debug, serde::Deserialize)]
pub struct Workflow {
    input: Option<String>,
    config: Config,
    tasks: Vec<Task>,
    steps: Vec<Edge>,
}

impl Workflow {
    pub fn new(input: Option<String>, tasks: Vec<Task>, steps: Vec<Edge>, config: Config) -> Self {
        Workflow {
            input,
            config,
            tasks,
            steps,
        }
    }

    pub fn new_from_json(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let workflow: Workflow = serde_json::from_reader(reader)?;
        Ok(workflow)
    }
}

impl Workflow {
    pub fn get_config(&self) -> &Config {
        &self.config
    }
    pub fn get_tasks(&self) -> &Vec<Task> {
        &self.tasks
    }
    pub fn get_workflow(&self) -> &Vec<Edge> {
        &self.steps
    }

    pub fn get_step(&self, index: u32) -> Option<&Edge> {
        self.steps.get(index as usize)
    }

    pub fn get_tasks_by_id(&self, task_id: &str) -> Option<&Task> {
        self.tasks.iter().find(|task| task.id == task_id)
    }
}
