# Ollama Workflows

Ollama Workflows is a framework that enables users to program LLMs through `workflows`. Workflows are JSON files with specific syntax that allow LLMs to divide complex tasks into smaller sub-tasks and complete them using tools and program memory. 

## Motivation

This framework was created to address the challenges faced when working with agentic frameworks and local LLMs. Some of the problems encountered include:

- The ReAct framework does not work well for local models and long-executed tasks.
- Different tasks require different types of optimizations and prompts.
- Local models with limited context length require a file system based on embeddings.

Existing agent libraries are extremely abstract, while the what we need are clear workflows and prompts. [Ollama](https://ollama.com/), in combination with [llama.cpp](https://github.com/ggerganov/llama.cpp), provides a great tool for running LLMs on consumer hardware. As people mostly use smaller LLMs without GPUs, task execution should be flexible and achievable in such cases.

## Design 

The design of Ollama Workflows is heavily inspired by the [`LLM OS`](https://x.com/karpathy/status/1723140519554105733) created by _Andrej Karpathy_. Ollama Workflows is designed to enable users to execute any task by providing task-specific programs, eliminating the need to use `rust` for programming.

Ollama Workflows does not require chat history and allows attaching specific prompts for each subtask along with I/O from memory. Learn how to design [workflows](docs/workflow.md)

### Key Components

- **Executor**: The Executor acts as the CPU, executing the workflow with operators. 

- **ProgramMemory**: Memory is used to pass data between steps of the workflow.

- **Workflows**: A Workflow is the instruction set for the program.

- **OpenAI Testing**: Ollama-workflows fully supports OpenAI, for faster prototyping and seamless testing for your workflows. 

For detailed documentation, please refer to the [documentation](docs/readme.md).

---

Created by @andthattoo
