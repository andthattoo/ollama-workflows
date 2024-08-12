# Ollama Workflows

Ollama Workflow is a Rust library that allows you to execute workflows using Large Language Models (LLMs). This README provides an overview of how things work.

## Executing Workflows

To execute a workflow, follow these steps:

1. Add the necessary dependencies to your Rust project:
   ```toml
   [dependencies]
   dotenv = "0.15.0"
   env_logger = "0.9.0"
   ollama_workflows = "0.1.0"
   tokio = { version = "1.0", features = ["full"] }
   ```

2. Import the required modules in your Rust code:
   ```rust
   use dotenv::dotenv;
   use env_logger::Env;
   use ollama_workflows::{Entry, Executor, Model, ProgramMemory, Workflow};
   ```

3. Load environment variables from a `.env` file (optional):
   ```rust
   dotenv().ok();
   ```

4. Initialize the logger:
   ```rust
   let env = Env::default().filter_or("LOG_LEVEL", "info");
   env_logger::Builder::from_env(env).init();
   ```

5. Create an instance of the `Executor` with the desired model:
   ```rust
   let exe = Executor::new(Model::Phi3Medium);
   ```

6. Load the workflow from a JSON file:
   ```rust
   let workflow = Workflow::new_from_json("path/to/your/workflow.json").unwrap();
   ```

7. Create an instance of `ProgramMemory`:
   ```rust
   let mut memory = ProgramMemory::new();
   ```

8. Prepare the input for the workflow (optional):
   ```rust
   let input = Entry::try_value_or_str("How does reiki work?");
   ```

9. Execute the workflow:
   ```rust
   exe.execute(Some(&input), workflow, &mut memory).await;
   ```

10. Access the final result from the `ProgramMemory`:
    ```rust
    println!("{:?}", memory.read(&"final_result".to_string()));
    ```

Note that workflows don't necessarily require inputs. When needed, the `Entry` enum can be used to input a string for the workflow.

## Designing Workflows

Workflows are defined using a JSON format. It has it's own syntax and logic just like a language.
Check the detailed documentation on how to design [workflows](workflow.md) and learn about the syntax.

### Operators

Workflows can use certain operators to perform specific tasks:

- `Generation`: Text generation with LLMs
- `FunctionCalling`: Function calling using LLMs. LLMs select the most suitable function based on the query, generate input parameters, and run the tool.
- `Check`: Compare two strings for equality (obsolete, replaced with conditions)
- `Search`: Perform vector search on `ProgramMemory`
- `End`: Ending operator

### Models

You can determine the model to use with the `Model` enum:

```rust
pub enum Model {
    // Ollama models
    /// [Nous's Hermes-2-Theta model](https://ollama.com/adrienbrault/nous-hermes2theta-llama3-8b), q8_0 quantized
    #[serde(rename = "adrienbrault/nous-hermes2theta-llama3-8b:q8_0")]
    NousTheta,
    #[default]
    /// [Microsoft's Phi3 Medium model](https://ollama.com/library/phi3:medium), q4_1 quantized
    #[serde(rename = "phi3:14b-medium-4k-instruct-q4_1")]
    Phi3Medium,
    /// [Microsoft's Phi3 Medium model, 128k context length](https://ollama.com/library/phi3:medium-128k), q4_1 quantized
    #[serde(rename = "phi3:14b-medium-128k-instruct-q4_1")]
    Phi3Medium128k,
    /// [Microsoft's Phi3 Mini model](https://ollama.com/library/phi3:3.8b), 3.8b parameters
    #[serde(rename = "phi3:3.8b")]
    Phi3Mini,
    /// [Ollama's Llama3.1 model](https://ollama.com/library/llama3.1:latest), 8B parameters
    #[serde(rename = "llama3.1:latest")]
    Llama3_1_8B,
    // OpenAI models
    /// [OpenAI's GPT-3.5 Turbo model](https://platform.openai.com/docs/models/gpt-3-5-turbo)
    #[serde(rename = "gpt-3.5-turbo")]
    GPT3_5Turbo,
    /// [OpenAI's GPT-4 Turbo model](https://platform.openai.com/docs/models/gpt-4-turbo-and-gpt-4)
    #[serde(rename = "gpt-4-turbo")]
    GPT4Turbo,
    /// [OpenAI's GPT-4o model](https://platform.openai.com/docs/models/gpt-4o)
    #[serde(rename = "gpt-4o")]
    GPT4o,
    /// [OpenAI's GPT-4o mini model](https://platform.openai.com/docs/models/gpt-4o-mini)
    #[serde(rename = "gpt-4o-mini")]
    GPT4oMini,
}
```

### Logs

Logs can be turned on/off through environment variables:
```bash
export LOG_LEVEL=info
```

Possible log levels are `info`, `warn`, and `off`, with decreasing amount of detail.

### Environment Variables

The `FunctionCalling` operator utilizes tools to execute certain subtasks. Some implemented tools require API keys, but ollama-workflow offers counterparts that don't require API keys.

OpenAI models also require API keys.

You can create a `.env` file to provide the following keys:

```bash
SERPER_API_KEY=[your SERPER_API_KEY]
JINA_API_KEY=[your JINA_API_KEY]
BROWSERLESS_TOKEN=[your BROWSERLESS_TOKEN]
OPENAI_API_KEY=[your OPENAI_API_KEY]
LOG_LEVEL="warn"