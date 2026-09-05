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
    let api_key =
        std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY environment variable not set");
    let client = Gemini::new(api_key)?;

    info!("advanced interaction example starting");

    let calc_tool = InteractionTool::function(
        "calculate",
        "Perform a mathematical calculation",
        json!({
            "type": "object",
            "properties": {
                "operation": { "type": "string", "description": "The operation: add, subtract, multiply, divide" },
                "a": { "type": "number" },
                "b": { "type": "number" }
            },
            "required": ["operation", "a", "b"]
        }),
    );

    let interaction = client
        .create_interaction()
        .with_model("gemini-flash-latest")
        .with_text("What is 42 times 12?")
        .with_system_instruction(
            "You are a helpful math assistant. Use the calculate tool for arithmetic.",
        )
        .with_tool(calc_tool)
        .with_temperature(0.7)
        .with_max_output_tokens(1000)
        .with_thinking_level(InteractionThinkingLevel::Low)
        .with_thinking_summaries(ThinkingSummaries::Auto)
        .with_stop_sequences(vec!["END".to_string()])
        .execute()
        .await?;

    for step in &interaction.steps {
        match step {
            Step::Thought { summary, .. } => {
                for content in summary {
                    let ThoughtSummaryContent::Text { text } = content;
                    info!(thought = text, "thought");
                }
            }
            Step::FunctionCall {
                name, arguments, ..
            } => {
                info!(function = name, args = %arguments, "function call");
            }
            Step::ModelOutput { content, .. } => {
                for c in content {
                    if let InteractionContent::Text { text, .. } = c {
                        info!(output = text, "model output");
                    }
                }
            }
            _ => {}
        }
    }

    if let Some(usage) = &interaction.usage {
        info!(
            input_tokens = usage.total_input_tokens,
            output_tokens = usage.total_output_tokens,
            thought_tokens = usage.total_thought_tokens,
            total_tokens = usage.total_tokens,
            "token usage"
        );
    }

    info!("advanced interaction example completed");
    Ok(())
}
