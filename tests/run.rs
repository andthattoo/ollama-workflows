use dotenv::dotenv;
use env_logger::Env;
use log::info;
use ollama_workflows::{Entry, Executor, Model, ProgramMemory, Workflow};

#[tokio::test]
async fn test_search_workflow() {
    dotenv().ok();
    let env = Env::default().filter_or("LOG_LEVEL", "info");
    env_logger::Builder::from_env(env).init();
    let exe = Executor::new(Model::Phi3Medium);
    let workflow = Workflow::new_from_json("./tests/test_workflows/search.json").unwrap();
    let mut memory = ProgramMemory::new();
    let input = Entry::try_value_or_str("How would does reiki work?");
    exe.execute(Some(&input), workflow, &mut memory).await;
}

#[tokio::test]
async fn test_question_generation() {
    dotenv().ok();
    let env = Env::default().filter_or("LOG_LEVEL", "info");
    env_logger::Builder::from_env(env).init();
    let exe = Executor::new(Model::GPT4oMini);
    let workflow = Workflow::new_from_json("./tests/test_workflows/questions.json").unwrap();
    let mut memory = ProgramMemory::new();
    let input = Entry::try_value_or_str("Best ZK Rollups to date, july 2024?");
    exe.execute(Some(&input), workflow, &mut memory).await;
}

#[tokio::test]
async fn test_search_workflow_openai() {
    dotenv().ok();
    let env = Env::default().filter_or("LOG_LEVEL", "info");
    env_logger::Builder::from_env(env).init();
    let exe = Executor::new(Model::GPT4oMini);
    let workflow = Workflow::new_from_json("./tests/test_workflows/search.json").unwrap();
    let mut memory = ProgramMemory::new();
    let input = Entry::try_value_or_str("Best ZK Rollups to date, july 2024?");
    exe.execute(Some(&input), workflow, &mut memory).await;
}

#[tokio::test]
async fn test_search_workflow_openai_all_tools() {
    dotenv().ok();
    let env = Env::default().filter_or("LOG_LEVEL", "info");
    env_logger::Builder::from_env(env).init();
    let exe = Executor::new(Model::GPT4oMini);
    let workflow = Workflow::new_from_json("./tests/test_workflows/all.json").unwrap();
    let mut memory = ProgramMemory::new();
    exe.execute(None, workflow, &mut memory).await;
}

#[tokio::test]
async fn test_post_process() {
    dotenv().ok();
    let env = Env::default().filter_or("LOG_LEVEL", "info");
    env_logger::Builder::from_env(env).init();
    let exe = Executor::new(Model::GPT4oMini);
    let workflow = Workflow::new_from_json("./tests/test_workflows/post_process.json").unwrap();
    let mut memory = ProgramMemory::new();
    let res = exe.execute(None, workflow, &mut memory).await;
    info!("Result: {:?}", res);
}

#[tokio::test]
async fn test_ticker_workflow_openai() {
    dotenv().ok();
    let env = Env::default().filter_or("LOG_LEVEL", "info");
    env_logger::Builder::from_env(env).init();
    let exe = Executor::new(Model::GPT4oMini);
    let workflow = Workflow::new_from_json("./tests/test_workflows/ticker.json").unwrap();
    let mut memory = ProgramMemory::new();
    exe.execute(None, workflow, &mut memory).await;
}

#[tokio::test]
async fn test_simple_workflow() {
    dotenv().ok();
    let env = Env::default().filter_or("LOG_LEVEL", "info");
    env_logger::Builder::from_env(env).init();
    let exe = Executor::new(Model::Phi3Medium);
    let workflow = Workflow::new_from_json("./tests/test_workflows/simple.json").unwrap();
    let mut memory = ProgramMemory::new();
    let input = Entry::try_value_or_str("How would does reiki work?");
    exe.execute(Some(&input), workflow, &mut memory).await;
}

/// Test the insert workflow
/// This workflow inserts a document into the file system.
#[tokio::test]
async fn test_insert_workflow_ollama() {
    dotenv().ok();
    let env = Env::default().filter_or("LOG_LEVEL", "debug");
    env_logger::Builder::from_env(env).init();
    let exe = Executor::new(Model::Phi3Medium);
    let workflow = Workflow::new_from_json("./tests/test_workflows/insert.json").unwrap();
    let mut memory = ProgramMemory::new();
    exe.execute(None, workflow, &mut memory).await;
}

#[tokio::test]
async fn test_insert_workflow_openai() {
    dotenv().ok();
    let env = Env::default().filter_or("LOG_LEVEL", "debug");
    env_logger::Builder::from_env(env).init();
    let exe = Executor::new(Model::GPT4oMini);
    let workflow = Workflow::new_from_json("./tests/test_workflows/insert.json").unwrap();
    let mut memory = ProgramMemory::new();
    exe.execute(None, workflow, &mut memory).await;
}

/// Test the user workflow
/// This workflow samples random variables and produces reviews based on sampled persona
#[tokio::test]
async fn test_user_workflow() {
    dotenv().ok();
    let env = Env::default().filter_or("LOG_LEVEL", "info");
    env_logger::Builder::from_env(env).init();
    let exe = Executor::new(Model::GPT4o);
    let workflow = Workflow::new_from_json("./tests/test_workflows/users.json").unwrap();
    let mut memory = ProgramMemory::new();
    exe.execute(None, workflow, &mut memory).await;
}
