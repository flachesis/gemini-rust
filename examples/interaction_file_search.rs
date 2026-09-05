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

    info!("file search interaction example starting");
    info!("note: set FILE_SEARCH_STORE env var or use default test store");

    let store_name = std::env::var("FILE_SEARCH_STORE")
        .unwrap_or_else(|_| "fileSearchStores/testmlbestpractices-ccpiz0vc7vix".to_string());

    let interaction = client
        .create_interaction()
        .with_model("gemini-flash-latest")
        .with_text("Search for documents about machine learning best practices")
        .with_file_search(vec![store_name])
        .execute()
        .await?;

    info!(response = interaction.output_text(), "file search response");

    for step in &interaction.steps {
        match step {
            Step::FileSearchCall { .. } => {
                info!("file search call initiated");
            }
            Step::FileSearchResult { .. } => {
                info!("file search result received");
            }
            _ => {}
        }
    }

    info!("file search example completed");
    Ok(())
}
