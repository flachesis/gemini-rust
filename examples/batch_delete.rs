//! Batch delete example
//!
//! This example demonstrates how to delete a batch operation after it has completed.
//! Deleting a batch operation removes its metadata from the system but does not cancel
//! a running operation.
//!
//! To run this example, you need to have a Gemini API key and an existing batch operation.
//! You can get an API key from the Google AI Studio.
//!
//! Once you have the API key, you can run this example by setting the `GEMINI_API_KEY`
//! and `BATCH_NAME` environment variables:
//!
//! ```sh
//! export GEMINI_API_KEY=your_api_key
//! export BATCH_NAME=your_batch_name
//! cargo run --package gemini-rust --example batch_delete
//! ```

use gemini_rust::{BatchStatus, Gemini};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the API key from the environment
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY not set");

    // Get the batch name from the environment
    let batch_name = env::var("BATCH_NAME").expect("BATCH_NAME not set");

    // Create a new Gemini client
    let gemini = Gemini::new(api_key).expect("unable to create Gemini API client");

    // Get the batch operation
    let batch = gemini.get_batch(&batch_name);

    // Check the batch status
    match batch.status().await {
        Ok(status) => {
            println!("Batch status: {:?}", status);

            // Only delete completed batches (succeeded, failed, cancelled, or expired)
            match status {
                BatchStatus::Succeeded { .. } | BatchStatus::Cancelled | BatchStatus::Expired => {
                    println!("Deleting batch operation...");
                    // We need to handle the std::result::Result<(), (Batch, Error)> return type
                    match batch.delete().await {
                        Ok(()) => println!("Batch deleted successfully!"),
                        Err((_batch, e)) => {
                            println!("Failed to delete batch: {}. You can retry with the returned batch.", e);
                            // Here you could retry: batch.delete().await, etc.
                        }
                    }
                }
                _ => {
                    println!("Batch is still running or pending. Use cancel() to stop it, or wait for completion before deleting.");
                }
            }
        }
        Err(e) => println!("Failed to get batch status: {}", e),
    }

    Ok(())
}
