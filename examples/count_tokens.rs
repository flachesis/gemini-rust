use display_error_chain::DisplayErrorChain;
use gemini_rust::{Gemini, Model};
use std::env;
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
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY not set");
    let client = Gemini::with_model(&api_key, Model::Gemini25Flash)?;

    info!("Preparing to count tokens for request...");

    let token_info = client.generate_content()
        .with_user_message("Hello, teacher!")                                   // 6 tokens
        .with_model_message("Hello! What would you like to learn about today?") // 11 tokens
        .with_user_message("Explain the theory of relativity in simple terms.") // 10 tokens
        .count_tokens()                                                         // Total tokens in AI Studio: 27 tokens
        .await?;

    info!(
        total_tokens = token_info.total_tokens,
        cached_tokens = ?token_info.cached_content_token_count,
        "Token count result for Gemini 2.5"
    );

    Ok(())
}