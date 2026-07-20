use display_error_chain::DisplayErrorChain;
use gemini_rust::prelude::*;
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

    info!("thinking levels example starting");

    for level in [
        InteractionThinkingLevel::Low,
        InteractionThinkingLevel::High,
    ] {
        info!(thinking.level = ?level, "sending request");

        let interaction = client
            .create_interaction()
            .with_model("gemini-flash-latest")
            .with_text("Explain Occam's razor and give a simple example")
            .with_thinking_level(level.clone())
            .with_thinking_summaries(ThinkingSummaries::Auto)
            .execute()
            .await?;

        let thoughts: Vec<String> = interaction
            .thoughts()
            .iter()
            .filter_map(|s| {
                if let Step::Thought { summary, .. } = s {
                    let texts: Vec<String> = summary
                        .iter()
                        .map(|c| {
                            let ThoughtSummaryContent::Text { text } = c;
                            text.clone()
                        })
                        .collect();
                    Some(texts)
                } else {
                    None
                }
            })
            .flatten()
            .collect();

        for (i, thought) in thoughts.iter().enumerate() {
            info!(thinking.level = ?level, thought.number = i + 1, thought = thought, "thought");
        }

        info!(thinking.level = ?level, answer = interaction.output_text(), "answer");

        if let Some(usage) = &interaction.usage {
            info!(
                thinking.level = ?level,
                thought_tokens = usage.total_thought_tokens,
                total_tokens = usage.total_tokens,
                "token usage"
            );
        }
    }

    info!("thinking levels example completed");
    Ok(())
}
