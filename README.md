# ollama-workflows

**

Create a .env file
```bash
SERPER_API_KEY=[your SERPER_API_KEY]
JINA_API_KEY=[your JINA_API_KEY]
BROWSERLESS_TOKEN=[your BROWSERLESS_TOKEN]
OPENAI_API_KEY=[your OPENAI_API_KEY]
LOG_LEVEL="warn"
```

Example run
```rust
use dotenv::dotenv;
use env_logger::Env;
use ollama_workflows::{Entry, Executor, Model, ProgramMemory, Workflow};

#[tokio::main]
async fn main() {
    dotenv().ok();
    let env = Env::default().filter_or("LOG_LEVEL", "info");
    env_logger::Builder::from_env(env).init();
    let exe = Executor::new(Model::Phi3Medium);
    let workflow = Workflow::new_from_json(
        "path/to/your/workflow.json",
    )
    .unwrap();
    let mut memory = ProgramMemory::new();
    let input = Entry::try_value_or_str("How would does reiki work?");
    exe.execute(Some(&input), workflow, &mut memory).await;
    println!("{:?}", memory.read(&"final_result".to_string()));
}
```
