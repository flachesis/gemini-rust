use gemini_rust::{FunctionCallingMode, FunctionDeclaration, Gemini, Schema, ThinkingConfig, Tool};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("GEMINI_API_KEY")?;
    let client = Gemini::pro(api_key)?;

    // Create a simple function tool
    let weather_function = FunctionDeclaration::new(
        "get_weather",
        "Get current weather for a location",
        Schema::object().with_property("location", Schema::string("City name"), true),
    );

    // Configure thinking to enable thoughtSignature
    let thinking_config = ThinkingConfig::new()
        .with_dynamic_thinking()
        .with_thoughts_included(true);

    let response = client
        .generate_content()
        .with_user_message("What's the weather like in Tokyo?")
        .with_tool(Tool::new(weather_function))
        .with_function_calling_mode(FunctionCallingMode::Auto)
        .with_thinking_config(thinking_config)
        .execute()
        .await?;

    // Check function calls and thought signatures
    let function_calls_with_thoughts = response.function_calls_with_thoughts();

    for (function_call, thought_signature) in function_calls_with_thoughts {
        println!("Function called: {}", function_call.name);
        println!("Arguments: {}", function_call.args);

        if let Some(signature) = thought_signature {
            println!("Thought signature present: {} characters", signature.len());
            println!(
                "Signature preview: {}...",
                &signature[..50.min(signature.len())]
            );
        } else {
            println!("No thought signature");
        }
    }

    Ok(())
}
