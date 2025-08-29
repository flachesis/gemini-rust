//! The Batch module for managing batch operations.
//!
//! For more information, see the official Google AI documentation:
//! - [Batch Mode Guide](https://ai.google.dev/gemini-api/docs/batch-mode)
//! - [Batch API Reference](https://ai.google.dev/api/batch-mode)
//!
//! # Design Note: Resource Management in Batch Operations
//!
//! The Batch API methods that consume the [`Batch`] struct (`cancel`, `delete`, `wait_for_completion`)
//! return `std::result::Result<T, (Self, crate::Error)>` instead of the crate's `Result<T>`.
//! This design follows patterns used in channel libraries (e.g., `std::sync::mpsc::Receiver`)
//! and provides two key benefits:
//!
//! 1. **Resource Safety**: Once a [`Batch`] is consumed by an operation, it cannot be used again,
//!    preventing invalid operations on deleted or canceled batches.
//!
//! 2. **Error Recovery**: If an operation fails due to transient network issues, both the
//!    [`Batch`] and error information are returned, allowing callers to retry the operation.
//!
//! ## Example usage:
//! ```rust,ignore
//! use gemini_rust::{Gemini, Message};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Gemini::new(std::env::var("GEMINI_API_KEY")?);
//!     let request = client.generate_content().with_user_message("Why is the sky blue?").build();
//!     let batch = client.batch_generate_content_sync().with_request(request).execute().await?;
//!
//!     match batch.delete().await {
//!         Ok(()) => println!("Batch deleted successfully!"),
//!         Err((batch, error)) => {
//!             println!("Failed to delete batch: {}", error);
//!             // Can retry: batch.delete().await
//!         }
//!     }
//!     Ok(())
//! }
//! ```

use std::{sync::Arc, time::Duration};
use tokio::time::sleep;

use crate::{
    client::GeminiClient,
    models::{BatchOperation, BatchStatus},
    Error, Result,
};

/// Represents a long-running batch operation, providing methods to manage its lifecycle.
///
/// A `Batch` object is a handle to a batch operation on the Gemini API. It allows you to
/// check the status, cancel the operation, or delete it once it's no longer needed.
pub struct Batch {
    /// The unique resource name of the batch operation, e.g., `operations/batch-xxxxxxxx`.
    pub name: String,
    client: Arc<GeminiClient>,
}

impl Batch {
    /// Creates a new Batch instance.
    pub(crate) fn new(name: String, client: Arc<GeminiClient>) -> Self {
        Self { name, client }
    }

    /// Returns the unique resource name of the batch operation.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Retrieves the current status of the batch operation by making an API call.
    ///
    /// This method provides a snapshot of the batch's state at a single point in time.
    pub async fn status(&self) -> Result<BatchStatus> {
        let operation: BatchOperation = self.client.get_batch_operation(&self.name).await?;
        BatchStatus::from_operation(operation)
    }

    /// Sends a request to the API to cancel the batch operation.
    ///
    /// Cancellation is not guaranteed to be instantaneous. The operation may continue to run for
    /// some time after the cancellation request is made.
    ///
    /// Consumes the batch. If cancellation fails, returns the batch and error information
    /// so it can be retried.
    pub async fn cancel(self) -> std::result::Result<(), (Self, crate::Error)> {
        match self.client.cancel_batch_operation(&self.name).await {
            Ok(()) => Ok(()),
            Err(e) => Err((self, e)),
        }
    }

    /// Deletes the batch operation resource from the server.
    ///
    /// Note: This method indicates that the client is no longer interested in the operation result.
    /// It does not cancel a running operation. To stop a running batch, use the `cancel` method.
    /// This method should typically be used after the batch has completed.
    ///
    /// Consumes the batch. If deletion fails, returns the batch and error information
    /// so it can be retried.
    pub async fn delete(self) -> std::result::Result<(), (Self, crate::Error)> {
        match self.client.delete_batch_operation(&self.name).await {
            Ok(()) => Ok(()),
            Err(e) => Err((self, e)),
        }
    }

    /// Waits for the batch operation to complete by periodically polling its status.
    ///
    /// This method polls the batch status with a specified delay until the operation
    /// reaches a terminal state (Succeeded, Failed, Cancelled, or Expired).
    ///
    /// Consumes the batch and returns the final status. If there's an error during polling,
    /// the batch is returned in the error variant so it can be retried.
    pub async fn wait_for_completion(
        self,
        delay: Duration,
    ) -> std::result::Result<BatchStatus, (Self, crate::Error)> {
        let batch_name = self.name.clone();
        loop {
            match self.status().await {
                Ok(status) => match status {
                    BatchStatus::Succeeded { .. } | BatchStatus::Cancelled => return Ok(status),
                    BatchStatus::Expired => {
                        return Err((self, Error::BatchExpired { name: batch_name }))
                    }
                    _ => sleep(delay).await,
                },
                Err(e) => match e {
                    Error::BatchFailed { .. } => return Err((self, e)),
                    _ => return Err((self, e)), // Return the batch and error for retry
                },
            }
        }
    }
}
