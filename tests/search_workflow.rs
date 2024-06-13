use dotenv::dotenv;
use env_logger::Env;
use ollama_workflows::{Entry, Executor, Model, ProgramMemory, Workflow};

#[tokio::test]
async fn test_search_workflow() {
    dotenv().ok();
    let env = Env::default().filter_or("LOG_LEVEL", "info");
    env_logger::Builder::from_env(env).init();
    let exe = Executor::new(Model::GPT3_5Turbo);
    let workflow = Workflow::new_from_json("./workflows/search.json").unwrap();
    let mut memory = ProgramMemory::new();
    let input = Entry::try_value_or_str("How would does reiki work?");
    exe.execute(Some(&input), workflow, &mut memory).await;
}
