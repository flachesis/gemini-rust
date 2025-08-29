//! The Batch module for managing batch operations.

use std::{sync::Arc, time::Duration};
use tokio::time::sleep;

use crate::{
    client::GeminiClient,
    models::{BatchOperation, BatchStatus},
    Error, Result,
};

/// Represents a long-running batch operation.
pub struct Batch {
    pub name: String,
    client: Arc<GeminiClient>,
}

impl Batch {
    /// Creates a new Batch instance.
    pub(crate) fn new(name: String, client: Arc<GeminiClient>) -> Self {
        Self { name, client }
    }

    /// Returns the name of the batch operation.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Retrieves the current status of the batch operation.
    pub async fn status(&self) -> Result<BatchStatus> {
        let operation: BatchOperation = self.client.get_batch_operation(&self.name).await?;
        BatchStatus::from_operation(operation)
    }

    /// Cancels the batch operation.
    pub async fn cancel(self) -> Result<()> {
        self.client.cancel_batch_operation(&self.name).await
    }

    /// Waits for the batch operation to complete.
    ///
    /// This method polls the batch status with a specified delay until the operation
    /// reaches a terminal state (Succeeded, Failed, Cancelled, or Expired).
    pub async fn wait_for_completion(self, delay: Duration) -> Result<BatchStatus> {
        loop {
            match self.status().await {
                Ok(status) => match status {
                    BatchStatus::Succeeded { .. } | BatchStatus::Cancelled => return Ok(status),
                    BatchStatus::Expired => {
                        return Err(Error::BatchExpired {
                            name: self.name.clone(),
                        })
                    }
                    _ => sleep(delay).await,
                },
                Err(e) => match e {
                    Error::BatchFailed { .. } => return Err(e),
                    _ => sleep(delay).await,
                },
            }
        }
    }
}
