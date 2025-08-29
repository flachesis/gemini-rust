use gemini_rust::{Content, Gemini, Part};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key from environment variable
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY environment variable not set");

    // This is equivalent to the curl example:
    // curl "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key=$YOUR_API_KEY" \
    //   -H 'Content-Type: application/json' \
    //   -X POST \
    //   -d '{
    //     "contents": [
    //       {
    //         "parts": [
    //           {
    //             "text": "Explain how AI works in a few words"
    //           }
    //         ]
    //       }
    //     ]
    //   }'

    // Create client - now using gemini-2.0-flash by default
    let client = Gemini::new(api_key);

    // Method 1: Using the high-level API (simplest approach)
    println!("--- Method 1: Using the high-level API ---");

    let response = client
        .generate_content()
        .with_user_message("Explain how AI works in a few words")
        .execute()
        .await?;

    println!("Response: {}", response.text());

    // Method 2: Using Content directly to match the curl example exactly
    println!("\n--- Method 2: Matching curl example structure exactly ---");

    // Create a content part that matches the JSON in the curl example
    let text_part = Part::Text {
        text: "Explain how AI works in a few words".to_string(),
        thought: None,
    };

    let content = Content {
        parts: Some(vec![text_part]),
        role: None,
    };

    // Add the content directly to the request
    // This exactly mirrors the JSON structure in the curl example
    let mut content_builder = client.generate_content();
    content_builder.contents.push(content);
    let response = content_builder.execute().await?;

    println!("Response: {}", response.text());

    Ok(())
}
