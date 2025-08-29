use std::sync::Arc;

use crate::{
    client::GeminiClient,
    models::{
        BatchConfig, BatchGenerateContentRequest, BatchGenerateContentResponse, BatchRequestItem,
        GenerateContentRequest, InputConfig, RequestMetadata, RequestsContainer,
    },
    Result,
};

/// Builder for synchronous batch content generation requests
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

    /// Set the display name for the batch request
    pub fn with_name(mut self, name: String) -> Self {
        self.display_name = name;
        self
    }

    /// Set the requests for the batch request
    pub fn with_requests(mut self, requests: Vec<GenerateContentRequest>) -> Self {
        self.requests = requests;
        self
    }

    /// Add a request to the batch request
    pub fn with_request(mut self, request: GenerateContentRequest) -> Self {
        self.requests.push(request);
        self
    }

    /// Build the batch request
    pub fn build(self) -> BatchGenerateContentRequest {
        let batch_requests: Vec<BatchRequestItem> = self
            .requests
            .into_iter()
            .enumerate()
            .map(|(i, request)| BatchRequestItem {
                request,
                metadata: Some(RequestMetadata {
                    key: format!("request-{}", i + 1),
                }),
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

    /// Execute the batch request
    pub async fn execute(self) -> Result<BatchGenerateContentResponse> {
        let client = self.client.clone();
        let request = self.build();
        client.batch_generate_content_sync(request).await
    }
}
