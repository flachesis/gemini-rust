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

use gemini_rust::{BatchResultItem, BatchStatus, Gemini, Message};
use std::time::Duration;

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
    let batch = gemini
        .batch_generate_content_sync()
        .with_request(request1)
        .with_request(request2)
        .execute()
        .await?;

    // Print the batch information
    println!("Batch created successfully!");
    println!("Batch Name: {}", batch.name());

    // Wait for the batch to complete
    println!("Waiting for batch to complete...");
    match batch.wait_for_completion(Duration::from_secs(5)).await {
        Ok(final_status) => {
            // Print the final status
            match final_status {
                BatchStatus::Succeeded { results } => {
                    println!("Batch succeeded!");
                    for item in results {
                        match item {
                            BatchResultItem::Success { key, response } => {
                                println!("--- Response for Key {} ---", key);
                                println!("{}", response.text());
                            }
                            BatchResultItem::Error { key, error } => {
                                println!("--- Error for Key {} ---", key);
                                println!("Code: {}, Message: {}", error.code, error.message);
                                if let Some(details) = &error.details {
                                    println!("Details: {}", details);
                                }
                            }
                        }
                    }
                }
                BatchStatus::Cancelled => {
                    println!("Batch was cancelled.");
                }
                BatchStatus::Expired => {
                    println!("Batch expired.");
                }
                _ => {
                    println!(
                        "Batch finished with an unexpected status: {:?}",
                        final_status
                    );
                }
            }
        }
        Err((_batch, e)) => {
            println!(
                "Batch failed: {}. You can retry with the returned batch.",
                e
            );
            // Here you could retry: batch.wait_for_completion(Duration::from_secs(5)).await, etc.
        }
    }

    Ok(())
}
