# Worklows

Core idea to [ollama-workflows](readme.md) is to program LLMs through `workflows`. Let us define the workflow structure

## Structure

A workflow JSON has following fields:

1. `name`: A string representing the name of the workflow.
2. `description`: A string describing the purpose of the workflow.
3. `config`: An object containing the configuration settings for the workflow.
4. `tasks`: An array of task objects defining the individual tasks in the workflow.
5. `steps`: An array of step objects specifying the order of execution and conditionals for the tasks.
6. `return_value`: A memory input to read and return the final designated value

## Config

The `config` field is an object with the following properties:

- `max_steps`: An integer specifying the maximum number of steps to execute before halting the program.
- `max_time`: An integer specifying the maximum execution time in seconds before halting the program.
- `tools`: An array of strings representing the set of tools to use in the workflow. Predefined tools include "browserless", "jina", "serper", "duckduckgo", "stock", and "scraper".
- `custom_tool`: An optional object describing a custom REST API request to serve as a tool.
- `max_tokens`: An optional integer specifying the maximum number of tokens for LLMs to generate per run.

### Tools
asdasd

## Tasks

The `tasks` field is an array of task objects. Tasks are designed to be the tasks to help reach your objective. Workflows help you outline the execution flow of each task. Each task object has the following properties:

- `id`: A unique string identifier for the task.
- `name`: A human-readable string name for the task.
- `description`: A string describing the task.
- `prompt`: A string prompt for the task, which can include placeholders for inputs (e.g., `{query}`).
- `inputs`: An array of input objects specifying [memory operations](#memory-operations) before the task.
- `operator`: An enum specifying the operator to be used for the task.
- `outputs`: An array of output objects specifying [memory operations](#memory-operations) after the task is finished.

Here's an example of a task in the `tasks` field. This task generates a web query based on an arbitrary input. The `prompt` section contains the prompt with two variables: `{query}` and `{history}`. The values corresponding to these variables in the prompt are obtained through the `inputs` field. The prompt ensures that the LLM generates a relevant web search query based on the user input and uses a history variable to avoid generating previously generated queries. Since workflows are state machines, a task can be executed multiple times.



```json
        {
            "id": "A",
            "name": "Web Search Query",
            "description": "Write a web search query to collect useful information for the given question",
            "prompt": "Write down a single search query to collect useful information to answer to given question. Be creative. Avoid asking previously asked questions, keep it concise and clear. \n###Query: {query} \n\n ###Previous Questions: {history} \n\n ###Search Query:",
            "inputs": [
                {
                    "name": "query",
                    "value": {
                        "type": "input",
                        "key": ""
                    },
                    "required": true
                },
                {
                    "name": "history",
                    "value": {
                        "type": "get_all",
                        "key": "history"
                    },
                    "required": false
                }
            ],
            "operator": "generation",
            "outputs": [
                {
                    "type": "write",
                    "key": "web_search_query",
                    "value": "__result"
                },
                {
                    "type": "push",
                    "key": "history",
                    "value": "__result"
                }
            ]
        }
```

### Inputs

Each input object in the `inputs` array has the following properties:

- `name`: A string representing the name of the input.
- `value`: An object specifying the input value type and associated data.
- `required`: A boolean indicating whether the input is required.

The `value` object has the following properties:

- `type`: An enum specifying the [memory operation](#memory-operations) as the input.
- `index`: An optional integer specifying the index for certain input value types. Used for `peek`
- `search_query`: An optional object specifying the search query for certain input value types.
- `key`: A string representing the key associated with the input value.

### Outputs

Each output object in the `outputs` array has the following properties:

- `type`: An enum specifying the [memory operation](#memory-operations) as the output type.
- `key`: A string representing the key associated with the output.
- `value`: A string representing the value to be written or pushed.

### Steps

The `steps` field is an array of step objects defining the order of execution and conditionals for the tasks. Each step object has the following properties:

- `source`: A string representing the source task ID.
- `target`: A string representing the target task ID.
- `condition` (optional): An object specifying a condition for the step.

The `condition` object has the following properties:

- `input`: An object specifying the input value for the condition.
- `expected`: A string representing the expected value for the condition.
- `expression`: An enum specifying the comparison expression (e.g., `Equal`, `NotEqual`, `Contains`, `NotContains`, `GreaterThan`, `LessThan`, `GreaterThanOrEqual`, `LessThanOrEqual`).
- `target_if_not`: A string representing the target task ID if the condition is not met.




## Example

Here's an example of a simple workflow JSON:

```json
{
    "name": "Simple",
    "description": "This is a simple workflow",
    "config":{
        "max_steps": 5,
        "max_time": 100,
        "tools": []
    },
    "tasks":[
        {
            "id": "A",
            "name": "Random Poem",
            "description": "Writes a poem about Cappadocia.",
            "prompt": "Please write a poem about Cappadocia.",
            "inputs":[],
            "operator": "generation",
            "outputs":[]
        },
        {
            "id": "__end",
            "name": "end",
            "description": "End of the task",
            "prompt": "",
            "inputs": [],
            "operator": "end",
            "outputs": []
        }
    ],
    "steps":[
        {
            "source":"A",
            "target":"__end"
        }
    ]
}
```

In this example, the workflow has two tasks: "Random Poem" and "end". The "Random Poem" task generates a poem about Cappadocia using the `generation` operator. The "end" task marks the end of the workflow.

The `steps` field specifies that the workflow starts with the "Random Poem" task (source: "A") and then proceeds to the "end" task (target: "__end"). 

More detailed examples can be found on [test workflows](../tests/test_workflows)

## Operators
- `generation`
- `function_calling`
- `check`
- `search`
- `sample`
- `end`.

## Memory Operations

The workflow supports various memory operations using a cache, stack, and file system.

- The cache is a key-value database for storing strings.
- The stack is a key-value database where the value is a vector of strings.
- The file system is a vector database that supports semantic search.

Memory operations are divided by I/O

**Inputs**
- `input`: Read user input
- `read`: Read from the cache.
- `pop`: Pop from the stack.
- `peek`: Peek from the stack.
- `get_all`: Get all entries from the stack.
- `size`: Get the size of the stack.
- `search`: Search the file system.

**Outputs**

- `write`: Write to the cache. 

Example:
```json
{
    "type": "write",
    "key": "web_search_query",
    "value": "__result"
},
```
This example writes the output of the task to cache. `__result` is the reserverd keyword for resulting value. Equivalent to
```json
"web_search_query": "the resulting value of the task"
``` 

- `push`: Push to the stack.
- `insert`: Insert into the file system.


These memory operations can be used in the `inputs` and `outputs` fields of the tasks to manipulate and access data during the workflow execution.