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
    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY environment variable not set");
    let client = Gemini::new(api_key)?;

    info!("interaction basic example starting");

    let interaction = client
        .create_interaction()
        .with_model("gemini-flash-latest")
        .with_text("Hello, how are you?")
        .execute()
        .await?;

    info!(response = interaction.output_text(), "simple response received");

    let interaction2 = client
        .create_interaction()
        .with_model("gemini-flash-latest")
        .with_system_instruction("You are a helpful assistant specializing in Rust programming.")
        .with_text("What makes Rust a good choice for systems programming?")
        .execute()
        .await?;

    info!(
        response = interaction2.output_text(),
        "response with system instruction received"
    );

    info!("interaction basic example completed");
    Ok(())
}
