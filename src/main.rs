use ollama_workflows::{Executor, Model, ProgramMemory, Workflow};
use parking_lot::Mutex;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let exe = Executor::new(Model::Phi3Medium);
    let workflow = Workflow::new_from_json("path/to/workflow.json").unwrap();
    //let memory = Arc::new(Mutex::new(ProgramMemory::new()));
    let mut memory = ProgramMemory::new();
    exe.execute(workflow, &mut memory).await;
}
