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

use futures::StreamExt;
use gemini_rust::Gemini;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt::init();
    // Get the API key from the environment
    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY not set");

    // Create a new Gemini client
    let gemini = Gemini::new(api_key).expect("unable to create Gemini API client");

    info!("listing all batch operations");

    // List all batch operations using the stream
    let stream = gemini.list_batches(5); // page_size of 5
    tokio::pin!(stream);

    while let Some(result) = stream.next().await {
        match result {
            Ok(operation) => {
                info!(
                    batch_name = operation.name,
                    state = ?operation.metadata.state,
                    created = %operation.metadata.create_time,
                    "batch operation found"
                );
            }
            Err(e) => {
                error!(error = ?e, "error fetching batch operation");
            }
        }
    }

    info!("finished listing batch operations");

    Ok(())
}
