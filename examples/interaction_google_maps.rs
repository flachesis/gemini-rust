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

    info!("google maps interaction example starting");

    let interaction = client
        .create_interaction()
        .with_model("gemini-flash-latest")
        .with_text("Find me coffee shops near Times Square, New York")
        .with_google_maps()
        .execute()
        .await?;

    info!(response = interaction.output_text(), "google maps response");

    for step in &interaction.steps {
        match step {
            Step::GoogleMapsCall { arguments, .. } => {
                info!(queries = ?arguments, "google maps call");
            }
            Step::GoogleMapsResult { result, .. } => {
                if let Some(places) = &result.places {
                    if let Some(ref p) = places.place_id {
                        info!(place_id = p, "found place");
                    }
                }
            }
            _ => {}
        }
    }

    info!("google maps example completed");
    Ok(())
}
