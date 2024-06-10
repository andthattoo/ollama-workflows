use env_logger::Env;
use ollama_workflows::{Entry, Executor, Model, ProgramMemory, Workflow};

#[tokio::main]
async fn main() {
    let env = Env::default().filter_or("LOG_LEVEL", "info");
    env_logger::Builder::from_env(env).init();
    let exe = Executor::new(Model::NousTheta);
    let workflow = Workflow::new_from_json(
        "/Users/kayaomers/Documents/firstbatch/ollama-workflows/my_workflows/search.json",
    )
    .unwrap();
    let mut memory = ProgramMemory::new();
    let input = Entry::from_str("What are the origins to Mevlana?");
    exe.execute(Some(&input), workflow, &mut memory).await;
}
