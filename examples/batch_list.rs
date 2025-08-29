//! Batch list example
//!
//! This example demonstrates how to list batch operations using a stream.
//!
//! To run this example, you need to have a Gemini API key.
//!
//! ```sh
//! export GEMINI_API_KEY=your_api_key
//! cargo run --package gemini-rust --example batch_list
//! ```

use futures::stream::StreamExt;
use gemini_rust::Gemini;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the API key from the environment
    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY not set");

    // Create a new Gemini client
    let gemini = Gemini::new(api_key);

    println!("Listing all batch operations...");

    // List all batch operations using the stream
    let stream = gemini.list_batches(5); // page_size of 5
    tokio::pin!(stream);

    while let Some(result) = stream.next().await {
        match result {
            Ok(operation) => {
                println!(
                    "  - Batch: {}, State: {:?}, Created: {}",
                    operation.name, operation.metadata.state, operation.metadata.create_time
                );
            }
            Err(e) => {
                eprintln!("Error fetching batch operation: {}", e);
            }
        }
    }

    println!("\nFinished listing operations.");

    Ok(())
}
