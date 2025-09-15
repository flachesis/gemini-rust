use gemini_rust::Gemini;
use std::env;
use tracing::info;

/// Example usage of Gemini API matching the curl example format
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt::init();
    // Replace with your actual API key
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY environment variable not set");

    // Create a Gemini client
    let gemini = Gemini::pro(api_key).expect("unable to create Gemini API client");

    // This example matches the exact curl request format:
    // curl "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key=$GEMINI_API_KEY" \
    //   -H 'Content-Type: application/json' \
    //   -d '{
    //     "system_instruction": {
    //       "parts": [
    //         {
    //           "text": "You are a cat. Your name is Neko."
    //         }
    //       ]
    //     },
    //     "contents": [
    //       {
    //         "parts": [
    //           {
    //             "text": "Hello there"
    //           }
    //         ]
    //       }
    //     ]
    //   }'
    let response = gemini
        .generate_content()
        .with_system_instruction("You are a cat. Your name is Neko.")
        .with_user_message("Hello there")
        .execute()
        .await?;

    // Log the response
    info!(response = response.text(), "gemini pro response received");

    Ok(())
}
