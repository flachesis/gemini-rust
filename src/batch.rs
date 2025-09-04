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

use snafu::{OptionExt, ResultExt, Snafu};
use std::{result::Result, sync::Arc};

use crate::{
    client::{Error as ClientError, GeminiClient},
    models::OperationError,
    models::{BatchOperation, OperationResult},
    BatchGenerateContentResponseItem, BatchResultItem, BatchState,
};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("batch '{name}' expired before finishing"))]
    BatchExpired {
        /// Batch name.
        name: String,
    },

    #[snafu(display("batch '{name}' failed"))]
    BatchFailed {
        source: OperationError,
        /// Batch name.
        name: String,
    },

    #[snafu(display("client invocation error"))]
    Client { source: Box<ClientError> },

    /// This error should never occur, as the Google API contract
    /// guarantees that a result will always be provided.
    ///
    /// I put it here anyway to avoid potential panic in case of
    /// Google's dishonesty or GCP internal errors.
    #[snafu(display("batch '{name}' completed but no result provided - API contract violation"))]
    MissingResult {
        /// Batch name.
        name: String,
    },
}

/// Represents the overall status of a batch operation.
#[derive(Debug, Clone, PartialEq)]
pub enum BatchStatus {
    /// The operation is waiting to be processed.
    Pending,
    /// The operation is currently being processed.
    Running {
        pending_count: i64,
        completed_count: i64,
        failed_count: i64,
        total_count: i64,
    },
    /// The operation has completed successfully.
    Succeeded { results: Vec<BatchResultItem> },
    /// The operation was cancelled by the user.
    Cancelled,
    /// The operation has expired.
    Expired,
}

impl TryFrom<BatchOperation> for BatchStatus {
    type Error = Error;

    fn try_from(operation: BatchOperation) -> Result<Self, Self::Error> {
        if operation.done {
            // According to Google API documentation, when done=true, result must be present
            let result = operation.result.context(MissingResultSnafu {
                name: operation.name.clone(),
            })?;

            match result {
                OperationResult::Failure { error } => Err(error).context(BatchFailedSnafu {
                    name: operation.name,
                }),
                OperationResult::Success { response } => {
                    let mut results: Vec<BatchResultItem> = response
                        .inlined_responses
                        .inlined_responses
                        .into_iter()
                        .map(|item| match item {
                            BatchGenerateContentResponseItem::Success { response, metadata } => {
                                BatchResultItem::Success {
                                    key: metadata.key,
                                    response,
                                }
                            }
                            BatchGenerateContentResponseItem::Error { error, metadata } => {
                                BatchResultItem::Error {
                                    key: metadata.key,
                                    error,
                                }
                            }
                        })
                        .collect();

                    // Sort results by key to ensure a consistent order.
                    results.sort_by_key(|item| {
                        let key_str = match item {
                            BatchResultItem::Success { key, .. } => key,
                            BatchResultItem::Error { key, .. } => key,
                        };
                        key_str.parse::<usize>().unwrap_or(usize::MAX)
                    });

                    // Handle terminal states based on metadata for edge cases
                    match operation.metadata.state {
                        BatchState::BatchStateCancelled => Ok(BatchStatus::Cancelled),
                        BatchState::BatchStateExpired => Ok(BatchStatus::Expired),
                        _ => Ok(BatchStatus::Succeeded { results }),
                    }
                }
            }
        } else {
            // The operation is still in progress.
            match operation.metadata.state {
                BatchState::BatchStatePending => Ok(BatchStatus::Pending),
                BatchState::BatchStateRunning => {
                    let total_count = operation.metadata.batch_stats.request_count;
                    let pending_count = operation
                        .metadata
                        .batch_stats
                        .pending_request_count
                        .unwrap_or(total_count);
                    let completed_count = operation
                        .metadata
                        .batch_stats
                        .completed_request_count
                        .unwrap_or(0);
                    let failed_count = operation
                        .metadata
                        .batch_stats
                        .failed_request_count
                        .unwrap_or(0);
                    Ok(BatchStatus::Running {
                        pending_count,
                        completed_count,
                        failed_count,
                        total_count,
                    })
                }
                // For non-running states when done=false, treat as pending
                _ => Ok(BatchStatus::Pending),
            }
        }
    }
}

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
    pub async fn status(&self) -> Result<BatchStatus, Error> {
        let operation: BatchOperation = self
            .client
            .get_batch_operation(&self.name)
            .await
            .map_err(Box::new)
            .context(ClientSnafu)?;

        BatchStatus::try_from(operation)
    }

    /// Sends a request to the API to cancel the batch operation.
    ///
    /// Cancellation is not guaranteed to be instantaneous. The operation may continue to run for
    /// some time after the cancellation request is made.
    ///
    /// Consumes the batch. If cancellation fails, returns the batch and error information
    /// so it can be retried.
    pub async fn cancel(self) -> Result<(), (Self, ClientError)> {
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
    pub async fn delete(self) -> Result<(), (Self, ClientError)> {
        match self.client.delete_batch_operation(&self.name).await {
            Ok(()) => Ok(()),
            Err(e) => Err((self, e)),
        }
    }
}
