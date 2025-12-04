use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::instrument;

use crate::client::{Error, GeminiClient};
use crate::file_search::model::{Operation, OperationResult};

/// A handle for monitoring long-running file upload/import operations.
///
/// Operations represent asynchronous processing tasks like file chunking,
/// embedding generation, and indexing. Use [`wait_until_done`](Self::wait_until_done)
/// to poll until completion.
#[derive(Debug, Clone)]
pub struct OperationHandle {
    client: Arc<GeminiClient>,
    operation: Operation,
}

impl OperationHandle {
    pub fn new(client: Arc<GeminiClient>, operation: Operation) -> Self {
        Self { client, operation }
    }

    pub fn name(&self) -> &str {
        &self.operation.name
    }

    pub fn is_done(&self) -> bool {
        self.operation.done.unwrap_or(false)
    }

    pub fn result(&self) -> Option<&OperationResult> {
        self.operation.result.as_ref()
    }

    #[instrument(skip_all, fields(operation.name = %self.operation.name))]
    pub async fn refresh(&mut self) -> Result<(), Error> {
        self.operation = self.client.get_operation(&self.operation.name).await?;
        Ok(())
    }

    #[instrument(skip_all, fields(
        operation.name = %self.operation.name,
        poll.interval.secs = interval.as_secs(),
        timeout.secs = timeout.as_ref().map(|d| d.as_secs()),
    ))]
    pub async fn wait_until_done(
        &mut self,
        interval: Duration,
        timeout: Option<Duration>,
    ) -> Result<(), Error> {
        let start = Instant::now();

        while !self.operation.done.unwrap_or(false) {
            if let Some(timeout) = timeout {
                if start.elapsed() >= timeout {
                    return Err(Error::OperationTimeout {
                        name: self.operation.name.clone(),
                    });
                }
            }

            tokio::time::sleep(interval).await;
            self.refresh().await?;
        }

        if let Some(OperationResult::Error { error }) = &self.operation.result {
            return Err(Error::OperationFailed {
                name: self.operation.name.clone(),
                code: error.code,
                message: error.message.clone(),
            });
        }

        Ok(())
    }

    /// Check if operation represents a file upload operation
    pub fn is_upload_operation(&self) -> bool {
        self.operation.name.contains("/upload/operations/")
    }

    /// Check if operation represents an import operation
    pub fn is_import_operation(&self) -> bool {
        self.operation.name.contains("/operations/") && !self.is_upload_operation()
    }

    /// Get the store name from operation name
    pub fn store_name(&self) -> Option<&str> {
        // Extract store name from operation name like "fileSearchStores/store-123/upload/operations/op-456"
        if let Some(start) = self.operation.name.find("fileSearchStores/") {
            let operation_path = &self.operation.name[start..];

            if self.is_upload_operation() {
                // For upload operations: "store-123/upload/operations/op-456"
                if let Some(upload_end) = operation_path.find("/upload/") {
                    Some(&self.operation.name[start..start + upload_end])
                } else {
                    None
                }
            } else {
                // For import operations: "store-123/operations/op-456"
                if let Some(end) = operation_path.find("/operations/") {
                    Some(&self.operation.name[start..start + end])
                } else {
                    None
                }
            }
        } else {
            None
        }
    }
}
