use env_logger::Env;
use ollama_workflows::{Entry, Executor, Model, ProgramMemory, Workflow};
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let env = Env::default().filter_or("LOG_LEVEL", "info");
    env_logger::Builder::from_env(env).init();
    let exe = Executor::new(Model::NousTheta);
    let workflow = Workflow::new_from_json(
        "/Users/kayaomers/Documents/firstbatch/ollama-workflows/my_workflows/search.json",
    )
    .unwrap();
    let mut memory = ProgramMemory::new();
    let input = Entry::from_str("How would does reiki work?");
    exe.execute(Some(&input), workflow, &mut memory).await;
    println!("{:?}", memory.read(&"final_result".to_string()));
}
