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

    info!("url context interaction example starting");

    let interaction = client
        .create_interaction()
        .with_model("gemini-flash-latest")
        .with_text("Summarize the main points from this article: https://en.wikipedia.org/wiki/Rust_(programming_language)")
        .with_url_context()
        .execute()
        .await?;

    info!(response = interaction.output_text(), "url context response");

    for step in &interaction.steps {
        if let Step::UrlContextResult { result, is_error, .. } = step {
            info!(
                url = result.url.as_deref().unwrap_or(""),
                status = ?result.status,
                is_error = is_error.unwrap_or(false),
                "url context result"
            );
        }
    }

    info!("url context example completed");
    Ok(())
}
