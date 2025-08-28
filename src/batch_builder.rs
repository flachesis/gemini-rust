use std::sync::Arc;

use crate::{
    client::GeminiClient,
    models::{BatchGenerateContentRequest, BatchGenerateContentResponse, GenerateContentRequest},
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
        BatchGenerateContentRequest {
            requests: self.requests,
        }
    }

    /// Execute the batch request
    pub async fn execute(self) -> Result<BatchGenerateContentResponse> {
        let client = self.client.clone();
        let request = self.build();
        client.batch_generate_content_sync(request).await
    }
}
