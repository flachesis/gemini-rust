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

    info!("tracing interaction example starting");

    let interaction = client
        .create_interaction()
        .with_model("gemini-flash-latest")
        .with_text("What is the capital of France?")
        .with_system_instruction("You are a concise geography expert.")
        .with_temperature(0.3)
        .execute()
        .await?;

    info!(
        response = interaction.output_text(),
        status = interaction.status.as_ref(),
        "interaction completed"
    );

    if let Some(usage) = &interaction.usage {
        info!(
            total_tokens = usage.total_tokens,
            input_tokens = usage.total_input_tokens,
            output_tokens = usage.total_output_tokens,
            "usage details"
        );
    }

    info!("tracing example completed — check the tracing output above for structured spans");
    Ok(())
}
