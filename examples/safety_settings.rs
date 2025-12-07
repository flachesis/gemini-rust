use display_error_chain::DisplayErrorChain;
use gemini_rust::{Gemini, HarmBlockThreshold, HarmCategory, SafetySetting};
use std::env;
use std::process::ExitCode;
use tracing::info;

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt().init();

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
    let client = Gemini::new(api_key)?;

    info!("Sending a request with explicit safety settings...");

    // Configure settings to block nothing
    let safety_settings = vec![
        SafetySetting {
            category: HarmCategory::HateSpeech,
            threshold: HarmBlockThreshold::BlockNone,
        },
        SafetySetting {
            category: HarmCategory::DangerousContent,
            threshold: HarmBlockThreshold::BlockNone,
        },
    ];

    let response = client
        .generate_content()
        .with_user_message("Tell me a scary story about a computer virus.")
        .with_safety_settings(safety_settings)
        .execute()
        .await?;

    info!(response = response.text(), "Response received safely");

    // Check if safety ratings are present in the response
    if let Some(candidate) = response.candidates.first() {
        if let Some(ratings) = &candidate.safety_ratings {
            info!(?ratings, "Safety ratings returned by API");
        }
    }

    Ok(())
}
