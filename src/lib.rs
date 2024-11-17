//! ollama-workflows
//!
//! This crate provides a flexible framework to create and execute workflows using the Ollama API.
//! It allows users to create workflows using JSON files and execute them using the provided Executor.
//! ### Example
//! ```rust
//! use dotenv::dotenv;
//! use env_logger::Env;
//! use ollama_workflows::{Entry, Executor, Model, ProgramMemory, Workflow};
//!
//! #[tokio::main]
//! async fn main() {
//!    dotenv().ok();
//!    let exe = setup_test($model).await;
//!    let workflow = Workflow::new_from_json($workflow).unwrap();
//!    let mut memory = ProgramMemory::new();
//!    let input = Entry::try_value_or_str($input);
//!    if let Err(e) = exe.execute(Some(&input), workflow, &mut memory).await {
//!        log::error!("Execution failed: {}", e);
//!    };
//! }
//!
//! ```
//! This crate provides a simple execution pipeline and enables users to create and execute workflows with JSON files.
//! Creating specific JSON for you purpose should suffice.
mod api_interface;
mod memory;
mod program;
mod tools;

pub use memory::types::Entry;
pub use memory::ProgramMemory;
pub use ollama_rs;
pub use program::{
    executor::Executor,
    models::{Model, ModelProvider},
    workflow::Workflow,
};

pub use program::atomics::{MessageInput, Task};
