use dotenv::dotenv;
use env_logger::Env;
use ollama_workflows::{Entry, Executor, Model, ProgramMemory, Workflow};

// Constants for workflow paths
const SEARCH_WORKFLOW_PATH: &str = "./tests/test_workflows/search.json";
const ALL_TOOLS_WORKFLOW_PATH: &str = "./tests/test_workflows/all.json";
const QUESTIONS_WORKFLOW_PATH: &str = "./tests/test_workflows/questions.json";
const POST_PROCESS_WORKFLOW_PATH: &str = "./tests/test_workflows/post_process.json";
const TICKER_WORKFLOW_PATH: &str = "./tests/test_workflows/ticker.json";
const SIMPLE_WORKFLOW_PATH: &str = "./tests/test_workflows/simple.json";
const INSERT_WORKFLOW_PATH: &str = "./tests/test_workflows/insert.json";
const USERS_WORKFLOW_PATH: &str = "./tests/test_workflows/users.json";
const CONTEXT_SIZE_WORKFLOW_PATH: &str = "./tests/test_workflows/context_size.json";
const CUSTOM_TOOL_HTTP_WORKFLOW_PATH: &str = "./tests/test_workflows/custom_tools_http.json";
const CUSTOM_TOOL_WORKFLOW_PATH: &str = "./tests/test_workflows/custom_tool.json";
const CODER_PATH: &str = "./tests/test_workflows/coding.json";
const BASE_PATH: &str = "./tests/test_workflows/base.json";
const STRUCTURED_PATH: &str = "./tests/test_workflows/structured_output.json";

async fn setup_test(model: Model) -> Executor {
    dotenv().ok();
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .filter_module("gem_rs", log::LevelFilter::Warn)
        .init();
    Executor::new(model)
}

macro_rules! workflow_test {
    ($name:ident, $model:expr, $workflow:expr, $input:expr) => {
        #[tokio::test]
        async fn $name() {
            let exe = setup_test($model).await;
            let workflow = Workflow::new_from_json($workflow).unwrap();
            let mut memory = ProgramMemory::new();
            let input = Entry::try_value_or_str($input);
            match exe.execute(Some(&input), &workflow, &mut memory).await {
                Ok(result) => {
                    log::info!("Execution completed successfully");
                    log::info!("Result: {:?}", result);
                }
                Err(e) => {
                    log::error!("Execution failed: {}", e);
                }
            };
        }
    };
    ($name:ident, $model:expr, $workflow:expr) => {
        #[tokio::test]
        async fn $name() {
            let exe = setup_test($model).await;
            let workflow = Workflow::new_from_json($workflow).unwrap();
            let mut memory = ProgramMemory::new();
            if let Err(e) = exe.execute(None, &workflow, &mut memory).await {
                log::error!("Execution failed: {}", e);
            };
        }
    };
}

// Search workflow tests
mod ticker_tests {
    use super::*;

    workflow_test!(
        ticker_workflow_openai,
        Model::GPT4oMini,
        TICKER_WORKFLOW_PATH
    );
}

mod simple_workflow_tests {
    use super::*;

    workflow_test!(
        gemini_simple_workflow,
        Model::Gemini15Flash,
        SIMPLE_WORKFLOW_PATH
    );

    workflow_test!(simple_coder, Model::Qwen2_5Coder1_5B, CODER_PATH);

    workflow_test!(o1_simple, Model::O1Mini, SIMPLE_WORKFLOW_PATH);
}

mod insert_workflow_tests {
    use super::*;

    workflow_test!(
        insert_workflow_ollama,
        Model::Phi3Medium,
        INSERT_WORKFLOW_PATH
    );
    workflow_test!(
        insert_workflow_openai,
        Model::GPT4oMini,
        INSERT_WORKFLOW_PATH
    );
}

mod user_workflow_tests {
    use super::*;

    workflow_test!(user_workflow, Model::GPT4o, USERS_WORKFLOW_PATH);
}

mod function_call_tests {
    use super::*;

    workflow_test!(
        gemini_function_call,
        Model::Gemini15Pro,
        SEARCH_WORKFLOW_PATH,
        "What is the most crowded street in Istanbul?"
    );

    workflow_test!(
        function_call_llama3_2_3b,
        Model::Llama3_2_3B,
        SEARCH_WORKFLOW_PATH,
        "What is the most famous street in Istanbul?"
    );
    workflow_test!(
        function_calling_openai,
        Model::GPT4o,
        SEARCH_WORKFLOW_PATH,
        "How many Hoodoo's are in Kapadokya?"
    );
    workflow_test!(
        function_calling_llama3_1,
        Model::Llama3_1_8B,
        SEARCH_WORKFLOW_PATH,
        "How many fairy chimneys are there in Cappadocia?"
    );
}

mod all_tools_workflow_tests {
    use super::*;

    workflow_test!(
        gemini_all_tools_workflow,
        Model::Gemini15Flash,
        ALL_TOOLS_WORKFLOW_PATH,
        "What's the weather like in New York and how does it affect the stock market?"
    );

    workflow_test!(
        openai_all_tools_workflow,
        Model::GPT4oMini,
        ALL_TOOLS_WORKFLOW_PATH,
        "What's the weather like in New York and how does it affect the stock market?"
    );
}

mod questions_workflow_tests {
    use super::*;

    workflow_test!(
        questions_workflow,
        Model::Phi3Medium,
        QUESTIONS_WORKFLOW_PATH,
        "Tell me about the history of artificial intelligence."
    );
}

mod post_process_workflow_tests {
    use super::*;

    workflow_test!(
        post_process_workflow,
        Model::Llama3_1_8B,
        POST_PROCESS_WORKFLOW_PATH,
        "Summarize the main plot points of Romeo and Juliet."
    );
}

mod context_size_tests {
    use super::*;

    workflow_test!(
        context_size_phi3_5,
        Model::Phi3_5Mini,
        CONTEXT_SIZE_WORKFLOW_PATH
    );
    workflow_test!(
        context_size_llama,
        Model::Llama3_1_8B,
        CONTEXT_SIZE_WORKFLOW_PATH
    );

    workflow_test!(
        context_size_gemini,
        Model::Gemini15Flash,
        CONTEXT_SIZE_WORKFLOW_PATH
    );
}

mod custom_tool_tests {
    use super::*;

    workflow_test!(
        gemini_custom_tool_workflow,
        Model::Gemini15Pro,
        CUSTOM_TOOL_WORKFLOW_PATH
    );

    workflow_test!(
        http_custom_tool_workflow,
        Model::Llama3_1_8B,
        CUSTOM_TOOL_HTTP_WORKFLOW_PATH
    );

    workflow_test!(
        gpt_custom_tool_workflow,
        Model::GPT4o,
        CUSTOM_TOOL_WORKFLOW_PATH
    );
}

mod base_workflow_tests {
    use super::*;

    workflow_test!(base_workflow, Model::Llama3_1_8BTextQ4KM, BASE_PATH);
}

mod base_structured_output_tests {
    use super::*;

    workflow_test!(llama_structured_output, Model::Llama3_1_8B, STRUCTURED_PATH);

    workflow_test!(openai_structured_output, Model::GPT4o, STRUCTURED_PATH);
}
