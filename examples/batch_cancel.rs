//! Example demonstrating how to cancel a batch operation when the user presses CTRL-C.
//!
//! This example shows:
//! 1. Creating a batch operation with multiple requests
//! 2. Setting up a signal handler for CTRL-C
//! 3. Starting the batch operation
//! 4. Canceling the batch when CTRL-C is pressed
//! 5. Properly handling the result

use gemini_rust::{Batch, BatchHandleError, BatchStatus, Gemini, Message};
use std::{env, sync::Arc, time::Duration};
use tokio::{signal, sync::Mutex};
use tracing::{error, info, warn};

/// Waits for the batch operation to complete by periodically polling its status.
///
/// This method polls the batch status with a specified delay until the operation
/// reaches a terminal state (Succeeded, Failed, Cancelled, or Expired).
///
/// Consumes the batch and returns the final status. If there's an error during polling,
/// the batch is returned in the error variant so it can be retried.
pub async fn wait_for_completion(
    batch: Batch,
    delay: Duration,
) -> Result<BatchStatus, (Batch, BatchHandleError)> {
    let batch_name = batch.name.clone();
    loop {
        match batch.status().await {
            Ok(status) => match status {
                BatchStatus::Succeeded { .. } | BatchStatus::Cancelled => return Ok(status),
                BatchStatus::Expired => {
                    return Err((batch, BatchHandleError::BatchExpired { name: batch_name }))
                }
                _ => tokio::time::sleep(delay).await,
            },
            Err(e) => match e {
                BatchHandleError::BatchFailed { .. } => return Err((batch, e)),
                _ => return Err((batch, e)), // Return the batch and error for retry
            },
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt::init();

    // Get the API key from the environment
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");

    // Create the Gemini client
    let gemini = Gemini::new(api_key).expect("unable to create Gemini API client");

    // Create a batch with multiple requests
    let mut batch_generate_content = gemini
        .batch_generate_content()
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
    info!(batch_name = batch.name(), "batch created successfully");
    info!("press ctrl-c to cancel the batch operation");

    // Wrap the batch in an Arc<Mutex<Option<Batch>>> to allow safe sharing
    let batch = Arc::new(Mutex::new(Some(batch)));
    let batch_clone = Arc::clone(&batch);

    // Spawn a task to handle CTRL-C
    let cancel_task = tokio::spawn(async move {
        // Wait for CTRL-C signal
        signal::ctrl_c().await.expect("Failed to listen for CTRL-C");
        info!("received ctrl-c, canceling batch operation");

        // Take the batch from the Option, leaving None.
        // The lock is released immediately after this block.
        let mut batch_to_cancel = batch_clone.lock().await;

        if let Some(batch) = batch_to_cancel.take() {
            // Cancel the batch operation
            match batch.cancel().await {
                Ok(()) => {
                    info!("batch canceled successfully");
                }
                Err((batch, e)) => {
                    warn!(error = %e, "failed to cancel batch, retrying");
                    // Retry once
                    match batch.cancel().await {
                        Ok(()) => {
                            info!("batch canceled successfully on retry");
                        }
                        Err((_, retry_error)) => {
                            error!(error = %retry_error, "failed to cancel batch even on retry");
                        }
                    }
                }
            }
        } else {
            info!("batch was already processed");
        }
    });

    // Wait for a short moment to ensure the cancel task is ready
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Wait for the batch to complete or be canceled
    if let Some(batch) = batch.lock().await.take() {
        info!("waiting for batch to complete or be canceled");
        match wait_for_completion(batch, Duration::from_secs(5)).await {
            Ok(final_status) => {
                // Cancel task is no longer needed since batch completed
                cancel_task.abort();

                info!(status = ?final_status, "batch completed");

                // Log details about the results
                match final_status {
                    gemini_rust::BatchStatus::Succeeded { .. } => {
                        info!("batch succeeded");
                    }
                    gemini_rust::BatchStatus::Cancelled => {
                        info!("batch was cancelled as requested");
                    }
                    gemini_rust::BatchStatus::Expired => {
                        warn!("batch expired");
                    }
                    _ => {
                        warn!("batch finished with unexpected status");
                    }
                }
            }
            Err((batch, e)) => {
                // This could happen if there was a network error while polling
                error!(error = %e, "error while waiting for batch completion");

                // Try one more time to get the status
                match batch.status().await {
                    Ok(status) => info!(status = ?status, "current batch status"),
                    Err(status_error) => {
                        error!(error = %status_error, "error getting final status")
                    }
                }
            }
        }
    }

    Ok(())
}
