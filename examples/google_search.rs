use gemini_rust::prelude::*;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key from environment variable
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY environment variable not set");

    // Create client
    let client = Gemini::new(api_key).expect("unable to create Gemini API client");

    println!("--- Google Search tool example ---");

    // Create a Google Search tool
    let google_search_tool = Tool::google_search();

    // Create a request with Google Search tool
    let response = client
        .generate_content()
        .with_user_message("What is the current Google stock price?")
        .with_tool(google_search_tool)
        .execute()
        .await?;

    println!("Response: {}", response.text());

    Ok(())
}
