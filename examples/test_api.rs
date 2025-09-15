use gemini_rust::Gemini;
use std::env;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt::init();

    let api_key = env::var("GEMINI_API_KEY")?;

    // Create client with the default model (gemini-2.0-flash)
    let client = Gemini::new(api_key).expect("unable to create Gemini API client");

    info!("sending request to gemini api");

    // Simple text completion with minimal content
    let response = client
        .generate_content()
        .with_user_message("Say hello")
        .execute()
        .await?;

    info!(response = response.text(), "api test completed");

    Ok(())
}
