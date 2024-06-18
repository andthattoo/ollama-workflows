# Worklows

Core idea to [ollama-workflows](readme.md) is to program LLMs through `workflows`. Let us define the workflow structure

## Structure

A workflow JSON has five main fields:

1. `name`: A string representing the name of the workflow.
2. `description`: A string describing the purpose of the workflow.
3. `config`: An object containing the configuration settings for the workflow.
4. `tasks`: An array of task objects defining the individual tasks in the workflow.
5. `steps`: An array of step objects specifying the order of execution and conditionals for the tasks.

## Config

The `config` field is an object with the following properties:

- `max_steps`: An integer specifying the maximum number of steps to execute before halting the program.
- `max_time`: An integer specifying the maximum execution time in seconds before halting the program.
- `tools`: An array of strings representing the set of tools to use in the workflow. Predefined tools include "browserless", "jina", "serper", "duckduckgo", "stock", and "scraper".
- `custom_tool`: An optional object describing a custom REST API request to serve as a tool.
- `max_tokens`: An optional integer specifying the maximum number of tokens for LLMs to generate per run.

## Tasks

The `tasks` field is an array of task objects. Each task object has the following properties:

- `id`: A unique string identifier for the task.
- `name`: A human-readable string name for the task.
- `description`: A string describing the task.
- `prompt`: A string prompt for the task, which can include placeholders for inputs (e.g., `{query}`).
- `inputs`: An array of input objects specifying memory operations before the task.
- `operator`: An enum specifying the operator to be used for the task (e.g., `Generation`, `FunctionCalling`, `Check`, `Search`, `End`).
- `outputs`: An array of output objects specifying memory operations after the task is finished.

### Inputs

Each input object in the `inputs` array has the following properties:

- `name`: A string representing the name of the input.
- `value`: An object specifying the input value type and associated data.
- `required`: A boolean indicating whether the input is required.

The `value` object has the following properties:

- `type`: An enum specifying the input value type (e.g., `Input`, `Read`, `Pop`, `Peek`, `GetAll`, `Size`, `String`).
- `index`: An optional integer specifying the index for certain input value types.
- `search_query`: An optional object specifying the search query for certain input value types.
- `key`: A string representing the key associated with the input value.

### Outputs

Each output object in the `outputs` array has the following properties:

- `type`: An enum specifying the output type (e.g., `Write`, `Insert`, `Push`).
- `key`: A string representing the key associated with the output.
- `value`: A string representing the value to be written or pushed.

## Steps

The `steps` field is an array of step objects defining the order of execution and conditionals for the tasks. Each step object has the following properties:

- `source`: A string representing the source task ID.
- `target`: A string representing the target task ID.
- `condition` (optional): An object specifying a condition for the step.

The `condition` object has the following properties:

- `input`: An object specifying the input value for the condition.
- `expected`: A string representing the expected value for the condition.
- `expression`: An enum specifying the comparison expression (e.g., `Equal`, `NotEqual`, `Contains`, `NotContains`, `GreaterThan`, `LessThan`, `GreaterThanOrEqual`, `LessThanOrEqual`).
- `target_if_not`: A string representing the target task ID if the condition is not met.