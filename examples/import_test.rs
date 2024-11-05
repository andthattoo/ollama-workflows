use dotenv::dotenv;
use env_logger::Env;
use ollama_workflows::{Executor, Model, ProgramMemory, Workflow};

fn main() {
    // Initialize environment
    dotenv().ok();
    let env = Env::default().filter_or("LOG_LEVEL", "info");
    env_logger::Builder::from_env(env).init();

    // Create runtime for async execution
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        // Initialize the executor with desired model
        let exe = Executor::new(Model::Phi3Medium);

        // Load workflow from JSON file
        let workflow = Workflow::new_from_json("./tests/test_workflows/simple.json").unwrap();

        // Initialize program memory
        let mut memory = ProgramMemory::new();

        // Execute workflow and handle the result
        match exe.execute(None, &workflow, &mut memory).await {
            Ok(result) => println!("Generated poem:\n{}", result),
            Err(err) => eprintln!("Error executing workflow: {:?}", err),
        }
    });
}
