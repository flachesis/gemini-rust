use display_error_chain::DisplayErrorChain;
use gemini_rust::prelude::*;
use serde_json::json;
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
    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY environment variable not set");
    let client = Gemini::new(api_key)?;

    info!("function calling example starting (stateful loop)");

    let get_weather = InteractionTool::function(
        "get_weather",
        "Get the current weather for a location",
        json!({
            "type": "object",
            "properties": {
                "location": { "type": "string", "description": "The city name" }
            },
            "required": ["location"]
        }),
    );

    let mut previous_id: Option<String> = None;
    let mut user_input = InteractionInput::text("What's the weather like in Tokyo?");

    loop {
        let mut builder = client
            .create_interaction()
            .with_model("gemini-flash-latest")
            .with_tool(get_weather.clone());

        if let Some(ref prev_id) = previous_id {
            builder = builder.with_previous_interaction(prev_id);
        }

        builder = match user_input {
            InteractionInput::Text(ref t) => builder.with_text(t.clone()),
            InteractionInput::StepArray(ref steps) => builder.with_step_input(steps.clone()),
            _ => builder,
        };

        let interaction = builder.execute().await?;

        let function_calls: Vec<_> = interaction.steps.iter().filter_map(|s| {
            if let Step::FunctionCall { name, arguments, id } = s {
                Some((name.clone(), arguments.clone(), id.clone()))
            } else {
                None
            }
        }).collect();

        if function_calls.is_empty() {
            info!(response = interaction.output_text(), "final response");
            break;
        }

        let mut results = Vec::new();
        for (name, args, call_id) in function_calls {
            info!(function_name = name, args = %args, "function call received");

            let location = args.get("location")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            let weather_result = json!({
                "temperature": 22,
                "unit": "celsius",
                "condition": "sunny",
                "location": location
            });

            results.push(Step::FunctionResult {
                name: Some(name),
                call_id,
                result: StepResult::from_json(weather_result),
                is_error: None,
            });
        }

        previous_id = interaction.id().map(|s| s.to_string());
        user_input = InteractionInput::step_array(results);
    }

    info!("function calling example completed");
    Ok(())
}
