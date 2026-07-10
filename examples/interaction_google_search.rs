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

    info!("google search interaction example starting");

    let interaction = client
        .create_interaction()
        .with_model("gemini-flash-latest")
        .with_text("What is the current Google stock price?")
        .with_google_search()
        .execute()
        .await?;

    info!(response = interaction.output_text(), "google search response");

    let citations = interaction.citations();
    if !citations.is_empty() {
        info!("citations found");
        for (i, citation) in citations.iter().enumerate() {
            match citation {
                Annotation::UrlCitation { url, title, start_index, end_index, .. } => {
                    info!(
                        citation.number = i + 1,
                        url = url.as_deref().unwrap_or(""),
                        title = title.as_deref().unwrap_or(""),
                        start = start_index,
                        end = end_index,
                        "URL citation"
                    );
                }
                Annotation::FileCitation { .. } => {
                    info!(citation.number = i + 1, "file citation");
                }
                Annotation::PlaceCitation { .. } => {
                    info!(citation.number = i + 1, "place citation");
                }
            }
        }
    }

    info!("google search example completed");
    Ok(())
}
