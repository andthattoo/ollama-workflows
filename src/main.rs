use ollama_workflows::core::workflow::Workflow;

fn main() {
    // JSON workflow definition and deserialization code remains the same
    // ...

    let workflow: Workflow = serde_json::from_str("").unwrap();
    let mut executor = WorkflowExecutor::new(workflow);
    executor.execute();
}