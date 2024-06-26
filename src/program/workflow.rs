use super::atomics::{Config, Edge, Task, TaskOutput};

/// Workflow serves as a container for the tasks and steps that make up a workflow.
#[derive(Debug, serde::Deserialize)]
pub struct Workflow {
    config: Config,
    tasks: Vec<Task>,
    steps: Vec<Edge>,
    return_value: TaskOutput,
}

impl Workflow {
    pub fn new(
        tasks: Vec<Task>,
        steps: Vec<Edge>,
        config: Config,
        return_value: TaskOutput,
    ) -> Self {
        Workflow {
            config,
            tasks,
            steps,
            return_value,
        }
    }

    /// Creates a new Workflow from a JSON file.
    pub fn new_from_json(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let workflow: Workflow = serde_json::from_reader(reader)?;
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
