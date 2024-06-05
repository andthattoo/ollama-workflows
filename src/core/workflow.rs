use super::atomics::{Task, Edge, Config};


#[derive(Debug, serde::Deserialize)]
pub struct Workflow {
    config: Config,
    tasks: Vec<Task>,
    steps: Vec<Edge>,
}

impl Workflow {
    pub fn new(config: Config, tasks: Vec<Task>, steps: Vec<Edge>) -> Self {
        Workflow {
            config,
            tasks,
            steps,
        }
    }
}

impl Default for Workflow {
    fn default() -> Self {
        Workflow {
            config: Config {
                max_steps: 100,
                max_time: 1000,
            },
            tasks: Vec::new(),
            steps: Vec::new(),
        }
    }
}

impl Workflow{
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