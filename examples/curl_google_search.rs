use gemini_rust::prelude::*;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key from environment variable
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY environment variable not set");

    println!("--- Curl equivalent with Google Search tool ---");

    // This is equivalent to the curl example:
    // curl "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key=$GEMINI_API_KEY" \
    //   -H "Content-Type: application/json" \
    //   -d '{
    //       "contents": [
    //           {
    //               "parts": [
    //                   {"text": "What is the current Google stock price?"}
    //               ]
    //           }
    //       ],
    //       "tools": [
    //           {
    //               "google_search": {}
    //           }
    //       ]
    //   }'

    // Create client
    let client = Gemini::new(api_key).expect("unable to create Gemini API client");

    // Create a content part that matches the JSON in the curl example
    let text_part = Part::Text {
        text: "What is the current Google stock price?".to_string(),
        thought: None,
        thought_signature: None,
    };

    let content = Content {
        parts: vec![text_part].into(),
        role: None,
    };

    // Create a Google Search tool
    let google_search_tool = Tool::google_search();

    // Add the content and tool directly to the request
    // This exactly mirrors the JSON structure in the curl example
    let mut content_builder = client.generate_content();
    content_builder.contents.push(content);
    content_builder = content_builder.with_tool(google_search_tool);

    let response = content_builder.execute().await?;

    println!("Response: {}", response.text());

    Ok(())
}
