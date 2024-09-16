use dotenv::dotenv;
use env_logger::Env;
use ollama_workflows::{Entry, Executor, Model, ProgramMemory, Workflow};

#[tokio::main]
async fn main() {
    dotenv().ok();
    let env = Env::default().filter_or("LOG_LEVEL", "info");
    env_logger::Builder::from_env(env).init();
    let exe = Executor::new(Model::GPT4Turbo);
    let workflow = Workflow::new_from_json("./workflows/search.json").unwrap();
    let mut memory = ProgramMemory::new();
    let input = Entry::try_value_or_str("How would does reiki work?");
    let return_value = exe.execute(Some(&input), workflow, &mut memory).await;
    match return_value {
        Ok(value) => println!("{}", value),
        Err(err) => eprintln!("Error: {:?}", err),
    }
}
