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
//!   let env = Env::default().filter_or("LOG_LEVEL", "info");
//!   env_logger.Builder::from_env(env).init();
//!   let exe = Executor::new(Model::Phi3Medium);
//!   let workflow = Workflow::new_from_json( "./workflows/search.json").unwrap();
//!   let mut memory = ProgramMemory::new();
//!   let input = Entry::try_value_or_str("How would does reiki work?");
//!   exe.execute(Some(&input), workflow, &mut memory).await;
//!   println!("{:?}", memory.read(&"final_result".to_string()));
//! }
//!
//! ```
//! This crate provides a simple execution pipeline and enables users to create and execute workflows with JSON files.
//! Creating specific JSON for you purpose should suffice.
mod memory;
mod program;
mod tools;

pub use memory::types::Entry;
pub use memory::ProgramMemory;
pub use program::{atomics::Model, executor::Executor, workflow::Workflow};
