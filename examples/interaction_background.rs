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
    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY environment variable not set");
    let client = Gemini::new(api_key)?;

    info!("background execution example starting");

    let interaction = client
        .create_interaction()
        .with_model("gemini-3.5-flash")
        .with_text("Write a detailed analysis of the impact of quantum computing on cryptography, covering current threats, post-quantum algorithms, and migration strategies.")
        .with_background()
        .execute()
        .await?;

    let id = interaction.id().expect("interaction should have an id");
    info!(
        interaction_id = id,
        status = interaction.status.as_ref(),
        "background interaction started"
    );

    let handle = client.interaction(id);
    let result = handle.poll_until_completed(Duration::from_secs(5)).await?;

    info!(
        status = result.status.as_ref(),
        "interaction completed"
    );

    info!(response = result.output_text(), "final response");

    if let Some(usage) = &result.usage {
        info!(
            total_tokens = usage.total_tokens,
            "token usage"
        );
    }

    info!("background execution example completed");
    Ok(())
}
