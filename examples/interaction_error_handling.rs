use display_error_chain::DisplayErrorChain;
use gemini_rust::prelude::*;
use std::process::ExitCode;
use tracing::{error, info};

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing::level_filters::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let api_key =
        std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY environment variable not set");

    match do_main(&api_key).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            let formatted = DisplayErrorChain::new(&e).to_string();
            error!(error = formatted, "request failed");
            ExitCode::FAILURE
        }
    }
}

async fn do_main(api_key: &str) -> Result<(), ClientError> {
    let client = Gemini::new(api_key)?;

    info!("sending interaction request");

    let interaction = client
        .create_interaction()
        .with_model("gemini-flash-latest")
        .with_text("Hello!")
        .execute()
        .await?;

    info!(response = interaction.output_text(), "response received");

    info!("attempting to get a non-existent interaction");

    match client.get_interaction("invalid-id-12345").await {
        Ok(_) => {
            info!("unexpectedly succeeded");
        }
        Err(e) => {
            let formatted = DisplayErrorChain::new(&e).to_string();
            info!(error = formatted, "expected error for invalid ID");
        }
    }

    info!("interaction error handling example completed");
    Ok(())
}
