use gemini_rust::Gemini;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("GEMINI_API_KEY")?;

    // Create client with the default model (gemini-2.0-flash)
    let client = Gemini::new(api_key).expect("unable to create Gemini API client");

    println!("Sending request to Gemini API...");

    // Simple text completion with minimal content
    let response = client
        .generate_content()
        .with_user_message("Say hello")
        .execute()
        .await?;

    println!("Response: {}", response.text());

    Ok(())
}
