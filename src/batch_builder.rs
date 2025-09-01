use std::sync::Arc;

use crate::{
    batch::Batch,
    client::GeminiClient,
    models::{
        BatchConfig, BatchGenerateContentRequest, BatchRequestItem, GenerateContentRequest,
        InputConfig, RequestMetadata, RequestsContainer,
    },
    Result,
};

/// A builder for creating and executing synchronous batch content generation requests.
///
/// This builder simplifies the process of constructing a batch request, allowing you to
/// add multiple `GenerateContentRequest` items and then execute them as a single
/// long-running operation.
pub struct BatchBuilder {
    client: Arc<GeminiClient>,
    display_name: String,
    requests: Vec<GenerateContentRequest>,
}

impl BatchBuilder {
    /// Create a new batch builder
    pub(crate) fn new(client: Arc<GeminiClient>) -> Self {
        Self {
            client,
            display_name: "RustBatch".to_string(),
            requests: Vec::new(),
        }
    }

    /// Sets the user-friendly display name for the batch request.
    pub fn with_name(mut self, name: String) -> Self {
        self.display_name = name;
        self
    }

    /// Sets all requests for the batch operation, replacing any existing requests.
    pub fn with_requests(mut self, requests: Vec<GenerateContentRequest>) -> Self {
        self.requests = requests;
        self
    }

    /// Adds a single `GenerateContentRequest` to the batch.
    pub fn with_request(mut self, request: GenerateContentRequest) -> Self {
        self.requests.push(request);
        self
    }

    /// Constructs the final `BatchGenerateContentRequest` from the builder's configuration.
    ///
    /// This method consumes the builder.
    pub fn build(self) -> BatchGenerateContentRequest {
        let batch_requests: Vec<BatchRequestItem> = self
            .requests
            .into_iter()
            .enumerate()
            .map(|(i, request)| BatchRequestItem {
                request,
                metadata: Some(RequestMetadata { key: i.to_string() }),
            })
            .collect();

        BatchGenerateContentRequest {
            batch: BatchConfig {
                display_name: self.display_name,
                input_config: InputConfig {
                    requests: RequestsContainer {
                        requests: batch_requests,
                    },
                },
            },
        }
    }

    /// Submits the batch request to the Gemini API and returns a `Batch` handle.
    ///
    /// This method consumes the builder and initiates the long-running batch operation.
    pub async fn execute(self) -> Result<Batch> {
        let client = self.client.clone();
        let request = self.build();
        let response = client.batch_generate_content_sync(request).await?;
        Ok(Batch::new(response.name, client))
    }
}
