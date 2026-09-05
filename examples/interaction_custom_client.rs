use display_error_chain::DisplayErrorChain;
use gemini_rust::{GeminiBuilder, Model};
use std::process::ExitCode;
use tracing::info;
use url::Url;

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

    let client = GeminiBuilder::new(&api_key)
        .with_model(Model::Custom("gemini-flash-latest".to_string()))
        .build()?;

    info!("custom client interaction example starting");

    let interaction = client
        .create_interaction()
        .with_text("Hello from a custom-configured client!")
        .execute()
        .await?;

    info!(response = interaction.output_text(), "response");

    let base_url = Url::parse("https://generativelanguage.googleapis.com/v1beta/")?;
    let client2 = GeminiBuilder::new(&api_key)
        .with_model(Model::Custom("gemini-pro-latest".to_string()))
        .with_base_url(base_url)
        .build()?;

    info!("using client with explicit base URL and pro model");

    let interaction2 = client2
        .create_interaction()
        .with_text("Explain quantum entanglement in one sentence.")
        .execute()
        .await?;

    info!(response = interaction2.output_text(), "pro response");

    info!("custom client example completed");
    Ok(())
}
