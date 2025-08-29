//! Example demonstrating how to cancel a batch operation when the user presses CTRL-C.
//!
//! This example shows:
//! 1. Creating a batch operation with multiple requests
//! 2. Setting up a signal handler for CTRL-C
//! 3. Starting the batch operation
//! 4. Canceling the batch when CTRL-C is pressed
//! 5. Properly handling the result

use gemini_rust::{Gemini, Message, Result};
use std::{env, sync::Arc, time::Duration};
use tokio::{signal, sync::Mutex};

#[tokio::main]
async fn main() -> Result<()> {
    // Get the API key from the environment
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");

    // Create the Gemini client
    let gemini = Gemini::new(api_key);

    // Create a batch with multiple requests
    let mut batch_generate_content = gemini
        .batch_generate_content_sync()
        .with_name("batch_cancel_example".to_string());

    // Add several requests to make the batch take some time to process
    for i in 1..=10 {
        let request = gemini
            .generate_content()
            .with_message(Message::user(format!(
                "Write a creative story about a robot learning to paint, part {}. Make it at least 100 words long.",
                i
            )))
            .build();

        batch_generate_content = batch_generate_content.with_request(request);
    }

    // Build and start the batch
    let batch = batch_generate_content.execute().await?;
    println!("Batch created successfully!");
    println!("Batch Name: {}", batch.name());
    println!("Press CTRL-C to cancel the batch operation...");

    // Wrap the batch in an Arc<Mutex<Option<Batch>>> to allow safe sharing
    let batch = Arc::new(Mutex::new(Some(batch)));
    let batch_clone = Arc::clone(&batch);

    // Spawn a task to handle CTRL-C
    let cancel_task = tokio::spawn(async move {
        // Wait for CTRL-C signal
        signal::ctrl_c().await.expect("Failed to listen for CTRL-C");
        println!("Received CTRL-C, canceling batch operation...");

        // Take the batch from the Option, leaving None.
        // The lock is released immediately after this block.
        let mut batch_to_cancel = batch_clone.lock().await;

        if let Some(batch) = batch_to_cancel.take() {
            // Cancel the batch operation
            match batch.cancel().await {
                Ok(()) => {
                    println!("Batch canceled successfully!");
                }
                Err((batch, e)) => {
                    println!("Failed to cancel batch: {}. Retrying...", e);
                    // Retry once
                    match batch.cancel().await {
                        Ok(()) => {
                            println!("Batch canceled successfully on retry!");
                        }
                        Err((_, retry_error)) => {
                            eprintln!("Failed to cancel batch even on retry: {}", retry_error);
                        }
                    }
                }
            }
        } else {
            println!("Batch was already processed.");
        }
    });

    // Wait for a short moment to ensure the cancel task is ready
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Wait for the batch to complete or be canceled
    if let Some(batch) = batch.lock().await.take() {
        println!("Waiting for batch to complete or be canceled...");
        match batch.wait_for_completion(Duration::from_secs(5)).await {
            Ok(final_status) => {
                // Cancel task is no longer needed since batch completed
                cancel_task.abort();

                println!("Batch completed with status: {:?}", final_status);

                // Print some details about the results
                match final_status {
                    gemini_rust::BatchStatus::Succeeded { .. } => {
                        println!("Batch succeeded!");
                    }
                    gemini_rust::BatchStatus::Cancelled => {
                        println!("Batch was cancelled as requested.");
                    }
                    gemini_rust::BatchStatus::Expired => {
                        println!("Batch expired.");
                    }
                    _ => {
                        println!("Batch finished with an unexpected status.");
                    }
                }
            }
            Err((batch, e)) => {
                // This could happen if there was a network error while polling
                println!("Error while waiting for batch completion: {}", e);

                // Try one more time to get the status
                match batch.status().await {
                    Ok(status) => println!("Current batch status: {:?}", status),
                    Err(status_error) => println!("Error getting final status: {}", status_error),
                }
            }
        }
    }

    Ok(())
}
