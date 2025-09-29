use display_error_chain::DisplayErrorChain;
use gemini_rust::{
    Content, FunctionCallingMode, FunctionDeclaration, FunctionParameters, Gemini, Message,
    PropertyDetails, Role, Tool,
};
use serde_json::json;
use std::env;
use std::process::ExitCode;
use tracing::info;

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing::level_filters::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    match do_main().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            let error_chain = DisplayErrorChain::new(e.as_ref());
            tracing::error!(error.debug = ?e, error.chained = %error_chain, "execution failed");
            ExitCode::FAILURE
        }
    }
}

async fn do_main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key from environment variable
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY environment variable not set");

    // Create client
    let client = Gemini::new(api_key).expect("unable to create Gemini API client");

    info!("starting tools example with multiple functions");

    // Define a weather function
    let get_weather = FunctionDeclaration::new(
        "get_weather",
        "Get the current weather for a location",
        FunctionParameters::object()
            .with_property(
                "location",
                PropertyDetails::string("The city and state, e.g., San Francisco, CA"),
                true,
            )
            .with_property(
                "unit",
                PropertyDetails::enum_type("The unit of temperature", ["celsius", "fahrenheit"]),
                false,
            ),
    );

    // Define a calculator function
    let calculate = FunctionDeclaration::new(
        "calculate",
        "Perform a calculation",
        FunctionParameters::object()
            .with_property(
                "operation",
                PropertyDetails::enum_type(
                    "The mathematical operation to perform",
                    ["add", "subtract", "multiply", "divide"],
                ),
                true,
            )
            .with_property("a", PropertyDetails::number("The first number"), true)
            .with_property("b", PropertyDetails::number("The second number"), true),
    );

    // Create a tool with multiple functions
    let tool = Tool::with_functions(vec![get_weather, calculate]);

    // Create a request with tool functions
    let response = client
        .generate_content()
        .with_system_prompt(
            "You are a helpful assistant that can check weather and perform calculations.",
        )
        .with_user_message("What's 42 times 12?")
        .with_tool(tool)
        .with_function_calling_mode(FunctionCallingMode::Any)
        .execute()
        .await?;

    // Process function calls
    if let Some(function_call) = response.function_calls().first() {
        info!(
            function_name = function_call.name,
            args = ?function_call.args,
            "function call received"
        );

        // Handle different function calls
        match function_call.name.as_str() {
            "calculate" => {
                let operation: String = function_call.get("operation")?;
                let a: f64 = function_call.get("a")?;
                let b: f64 = function_call.get("b")?;

                info!(
                    a = a,
                    operation = operation,
                    b = b,
                    "performing calculation"
                );

                let result = match operation.as_str() {
                    "add" => a + b,
                    "subtract" => a - b,
                    "multiply" => a * b,
                    "divide" => a / b,
                    _ => panic!("Unknown operation"),
                };

                let function_response = json!({
                    "result": result,
                })
                .to_string();

                // Based on the curl example, we need to structure the conversation properly:
                // 1. A user message with the original query
                // 2. A model message containing the function call
                // 3. A user message containing the function response

                // Construct conversation following the exact curl pattern
                let mut conversation = client.generate_content();

                // 1. Add user message with original query and system prompt
                conversation = conversation
                    .with_system_prompt("You are a helpful assistant that can check weather and perform calculations.")
                    .with_user_message("What's 42 times 12?");

                // 2. Create model content with function call
                let model_content = Content::function_call((*function_call).clone());

                // Add as model message
                let model_message = Message {
                    content: model_content,
                    role: Role::Model,
                };
                conversation = conversation.with_message(model_message);

                // 3. Add user message with function response
                conversation =
                    conversation.with_function_response_str("calculate", function_response)?;

                // Execute the request
                let final_response = conversation.execute().await?;

                info!(response = final_response.text(), "final response received");
            }
            "get_weather" => {
                let location: String = function_call.get("location")?;
                let unit = function_call
                    .get::<String>("unit")
                    .unwrap_or_else(|_| String::from("celsius"));

                info!(location = location, unit = unit, "weather request received");

                let weather_response = json!({
                    "temperature": 22,
                    "unit": unit,
                    "condition": "sunny"
                })
                .to_string();

                // Based on the curl example, we need to structure the conversation properly:
                // 1. A user message with the original query
                // 2. A model message containing the function call
                // 3. A user message containing the function response

                // Construct conversation following the exact curl pattern
                let mut conversation = client.generate_content();

                // 1. Add user message with original query and system prompt
                conversation = conversation
                    .with_system_prompt("You are a helpful assistant that can check weather and perform calculations.")
                    .with_user_message("What's 42 times 12?");

                // 2. Create model content with function call
                let model_content = Content::function_call((*function_call).clone());

                // Add as model message
                let model_message = Message {
                    content: model_content,
                    role: Role::Model,
                };
                conversation = conversation.with_message(model_message);

                // 3. Add user message with function response
                conversation =
                    conversation.with_function_response_str("get_weather", weather_response)?;

                // Execute the request
                let final_response = conversation.execute().await?;

                info!(response = final_response.text(), "final response received");
            }
            _ => info!(function_name = function_call.name, "unknown function call"),
        }
    } else {
        info!("no function calls in response");
        info!(response = response.text(), "direct response received");
    }

    Ok(())
}
