use display_error_chain::DisplayErrorChain;
use gemini_rust::{
    FunctionCallingMode, FunctionDeclaration, FunctionParameters, Gemini, PropertyDetails,
    ThinkingConfig, Tool,
};
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
    let api_key = env::var("GEMINI_API_KEY")?;
    let client = Gemini::pro(api_key)?;

    // Create a simple function tool
    let weather_function = FunctionDeclaration::new(
        "get_weather",
        "Get current weather for a location",
        FunctionParameters::object().with_property(
            "location",
            PropertyDetails::string("City name"),
            true,
        ),
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
        info!(function_name = function_call.name, args = ?function_call.args, "function called");

        if let Some(signature) = thought_signature {
            info!(
                signature_length = signature.len(),
                preview = &signature[..50.min(signature.len())],
                "thought signature present"
            );
        } else {
            info!("no thought signature");
        }
    }

    Ok(())
}
