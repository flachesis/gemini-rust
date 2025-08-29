//! Batch generate content example
//!
//! This example demonstrates how to use the synchronous batch generate content API to generate content for multiple requests at once.
//!
//! To run this example, you need to have a Gemini API key. You can get one from the Google AI Studio.
//!
//! Once you have the API key, you can run this example by setting the `GEMINI_API_KEY` environment variable:
//!
//! ```sh
//! export GEMINI_API_KEY=your_api_key
//! cargo run --package gemini-rust --example batch_generate
//! ```

use gemini_rust::{Gemini, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the API key from the environment
    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY not set");

    // Create a new Gemini client
    let gemini = Gemini::new(api_key);

    // Create the first request
    let request1 = gemini
        .generate_content()
        .with_message(Message::user("What is the meaning of life?"))
        .build();

    // Create the second request
    let request2 = gemini
        .generate_content()
        .with_message(Message::user("What is the best programming language?"))
        .build();

    // Create the batch request
    let batch_response = gemini
        .batch_generate_content_sync()
        .with_request(request1)
        .with_request(request2)
        .execute()
        .await?;

    // Print the batch information
    println!("Batch created successfully!");
    println!("Batch ID: {}", batch_response.name);
    println!("State: {}", batch_response.metadata.state);
    println!(
        "Request count: {}",
        batch_response.metadata.batch_stats.request_count
    );

    if batch_response.metadata.state == "BATCH_STATE_PENDING" {
        println!("Batch is currently processing. You would need to poll the batch status to get results.");
        println!("Note: This is an async operation that may take some time to complete.");
    }

    Ok(())
}
