use display_error_chain::DisplayErrorChain;
use gemini_rust::prelude::*;
use std::process::ExitCode;
use std::time::Duration;
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

    info!("deep research agent example starting");

    let interaction = client
        .create_interaction()
        .with_agent("deep-research-preview-04-2026")
        .with_text("Research the current state of quantum computing: key milestones, major players, and future predictions for 2026-2030.")
        .with_agent_config(AgentConfig::DeepResearch {
            thinking_summaries: Some(ThinkingSummaries::Auto),
            visualization: Some(Visualization::Auto),
            collaborative_planning: Some(false),
            enable_bigquery_tool: None,
        })
        .with_background()
        .execute()
        .await?;

    let id = interaction.id().expect("interaction should have an id");
    info!(
        interaction_id = id,
        status = interaction.status.as_ref(),
        "deep research started (this may take several minutes)"
    );

    let handle = client.interaction(id);
    let result = handle.poll_until_completed(Duration::from_secs(10)).await?;

    info!(status = result.status.as_ref(), "deep research completed");

    info!(response = result.output_text(), "research output");

    let thoughts = result.thoughts();
    if !thoughts.is_empty() {
        info!("thought summaries");
        for (i, thought) in thoughts.iter().enumerate() {
            if let Step::Thought { summary, .. } = thought {
                for content in summary {
                    let ThoughtSummaryContent::Text { text } = content;
                    info!(thought.number = i + 1, thought = text, "thought summary");
                }
            }
        }
    }

    info!("deep research example completed");
    Ok(())
}
