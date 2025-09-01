use crate::{
    batch_builder::BatchBuilder,
    content_builder::ContentBuilder,
    embed_builder::EmbedBuilder,
    models::{
        BatchContentEmbeddingResponse, BatchEmbedContentsRequest, BatchGenerateContentRequest,
        BatchGenerateContentResponse, BatchOperation, ContentEmbeddingResponse,
        EmbedContentRequest, GenerateContentRequest, GenerationResponse, ListBatchesResponse,
    },
    Batch, Error, Result,
};
use futures::stream::Stream;
use reqwest::Client;
use serde_json::Value;
use std::{pin::Pin, sync::Arc};
use url::Url;

const DEFAULT_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/";
const DEFAULT_MODEL: &str = "models/gemini-2.5-flash";

/// Internal client for making requests to the Gemini API
pub(crate) struct GeminiClient {
    http_client: Client,
    api_key: String,
    pub model: String,
    base_url: String,
}

impl GeminiClient {
    /// Create a new client
    #[allow(dead_code)]
    fn new(api_key: impl Into<String>, model: String) -> Self {
        Self::with_base_url(api_key, model, DEFAULT_BASE_URL.to_string())
    }

    /// Create a new client with custom base URL
    fn with_base_url(api_key: impl Into<String>, model: String, base_url: String) -> Self {
        Self {
            http_client: Client::new(),
            api_key: api_key.into(),
            model,
            base_url,
        }
    }

    /// Generate content
    pub(crate) async fn generate_content_raw(
        &self,
        request: GenerateContentRequest,
    ) -> Result<GenerationResponse> {
        let url = self.build_url("generateContent")?;

        let response = self.http_client.post(url).json(&request).send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(Error::ApiError {
                status_code: status.as_u16(),
                message: error_text,
            });
        }

        let response = response.json().await?;

        Ok(response)
    }

    /// Generate content with streaming
    pub(crate) async fn generate_content_stream(
        &self,
        request: GenerateContentRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<GenerationResponse>> + Send>>> {
        let url = self.build_url("streamGenerateContent")?;

        let response = self.http_client.post(url).json(&request).send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(Error::ApiError {
                status_code: status.as_u16(),
                message: error_text,
            });
        }

        // Get the full response as bytes and parse as JSON array
        let bytes = response.bytes().await?;
        let text = String::from_utf8_lossy(&bytes);

        // The Gemini API returns a JSON array format like: [{json1}, {json2}, {json3}]
        let responses: Vec<Result<GenerationResponse>> =
            match serde_json::from_str::<Vec<GenerationResponse>>(&text) {
                Ok(json_array) => json_array.into_iter().map(Ok).collect(),
                Err(e) => {
                    vec![Err(Error::JsonError(e))]
                }
            };

        let stream = futures::stream::iter(responses);
        Ok(Box::pin(stream))
    }

    /// Embed content
    pub(crate) async fn embed_content(
        &self,
        request: EmbedContentRequest,
    ) -> Result<ContentEmbeddingResponse> {
        let value = self.post_json(request, "embedContent").await?;
        let response = serde_json::from_value::<ContentEmbeddingResponse>(value)?;

        Ok(response)
    }

    /// Batch Embed content
    pub(crate) async fn embed_content_batch(
        &self,
        request: BatchEmbedContentsRequest,
    ) -> Result<BatchContentEmbeddingResponse> {
        let value = self.post_json(request, "batchEmbedContents").await?;
        let response = serde_json::from_value::<BatchContentEmbeddingResponse>(value)?;

        Ok(response)
    }

    /// Synchronous Batch Generate content
    pub(crate) async fn batch_generate_content_sync(
        &self,
        request: BatchGenerateContentRequest,
    ) -> Result<BatchGenerateContentResponse> {
        let value = self.post_json(request, "batchGenerateContent").await?;
        let response = serde_json::from_value::<BatchGenerateContentResponse>(value)?;
        Ok(response)
    }

    /// Get a batch operation
    pub(crate) async fn get_batch_operation<T: serde::de::DeserializeOwned>(
        &self,
        name: &str,
    ) -> Result<T> {
        let url = self.build_batch_url(name, None)?;
        let response = self.http_client.get(url).send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(Error::ApiError {
                status_code: status.as_u16(),
                message: error_text,
            });
        }

        let response = response.json().await?;
        Ok(response)
    }

    /// List batch operations
    pub(crate) async fn list_batch_operations(
        &self,
        page_size: Option<u32>,
        page_token: Option<String>,
    ) -> Result<ListBatchesResponse> {
        let mut url = self.build_batch_url("batches", None)?;

        if let Some(size) = page_size {
            url.query_pairs_mut()
                .append_pair("pageSize", &size.to_string());
        }
        if let Some(token) = page_token {
            url.query_pairs_mut().append_pair("pageToken", &token);
        }

        let response = self.http_client.get(url).send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(Error::ApiError {
                status_code: status.as_u16(),
                message: error_text,
            });
        }

        let response = response.json().await?;
        Ok(response)
    }

    /// Cancel a batch operation
    pub(crate) async fn cancel_batch_operation(&self, name: &str) -> Result<()> {
        let url = self.build_batch_url(name, Some("cancel"))?;
        let response = self
            .http_client
            .post(url)
            .json(&serde_json::json!({}))
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(Error::ApiError {
                status_code: status.as_u16(),
                message: error_text,
            });
        }

        Ok(())
    }

    /// Delete a batch operation
    pub(crate) async fn delete_batch_operation(&self, name: &str) -> Result<()> {
        let url = self.build_batch_url(name, None)?;
        let response = self.http_client.delete(url).send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(Error::ApiError {
                status_code: status.as_u16(),
                message: error_text,
            });
        }

        Ok(())
    }

    /// Post JSON to an endpoint
    async fn post_json<T: serde::Serialize>(&self, request: T, endpoint: &str) -> Result<Value> {
        let url = self.build_url(endpoint)?;

        let response = self.http_client.post(url).json(&request).send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(Error::ApiError {
                status_code: status.as_u16(),
                message: error_text,
            });
        }

        let response = response.json().await?;
        Ok(response)
    }

    /// Build a URL for the API
    fn build_url(&self, endpoint: &str) -> Result<Url> {
        let url_str = format!(
            "{}{}:{}?key={}",
            self.base_url, self.model, endpoint, self.api_key
        );
        Url::parse(&url_str).map_err(|e| Error::RequestError(e.to_string()))
    }

    /// Build a URL for a batch operation
    fn build_batch_url(&self, name: &str, action: Option<&str>) -> Result<Url> {
        let action_suffix = action.map_or("".to_string(), |a| format!(":{}", a));
        let url_str = format!(
            "{}{}{}?key={}",
            self.base_url, name, action_suffix, self.api_key
        );
        Url::parse(&url_str).map_err(|e| Error::RequestError(e.to_string()))
    }
}

/// Client for the Gemini API
#[derive(Clone)]
pub struct Gemini {
    client: Arc<GeminiClient>,
}

impl Gemini {
    /// Create a new client with the specified API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::with_model(api_key, DEFAULT_MODEL.to_string())
    }

    /// Create a new client for the Gemini Pro model
    pub fn pro(api_key: impl Into<String>) -> Self {
        Self::with_model(api_key, "models/gemini-2.5-pro".to_string())
    }

    /// Create a new client with the specified API key and model
    pub fn with_model(api_key: impl Into<String>, model: String) -> Self {
        Self::with_model_and_base_url(api_key, model, DEFAULT_BASE_URL.to_string())
    }

    /// Create a new client with custom base URL
    pub fn with_base_url(api_key: impl Into<String>, base_url: String) -> Self {
        Self::with_model_and_base_url(api_key, DEFAULT_MODEL.to_string(), base_url)
    }

    /// Create a new client with the specified API key, model, and base URL
    pub fn with_model_and_base_url(
        api_key: impl Into<String>,
        model: String,
        base_url: String,
    ) -> Self {
        let client = GeminiClient::with_base_url(api_key.into(), model, base_url);
        Self {
            client: Arc::new(client),
        }
    }

    /// Start building a content generation request
    pub fn generate_content(&self) -> ContentBuilder {
        ContentBuilder::new(self.client.clone())
    }

    /// Start building a content generation request
    pub fn embed_content(&self) -> EmbedBuilder {
        EmbedBuilder::new(self.client.clone())
    }

    /// Start building a synchronous batch content generation request
    pub fn batch_generate_content_sync(&self) -> BatchBuilder {
        BatchBuilder::new(self.client.clone())
    }

    /// Get a handle to a batch operation by its name.
    pub fn get_batch(&self, name: &str) -> Batch {
        Batch::new(name.to_string(), self.client.clone())
    }

    /// Lists batch operations.
    ///
    /// This method returns a stream that handles pagination automatically.
    pub fn list_batches(
        &self,
        page_size: impl Into<Option<u32>>,
    ) -> impl Stream<Item = Result<BatchOperation>> + Send {
        let client = self.client.clone();
        let page_size = page_size.into();
        async_stream::try_stream! {
            let mut page_token: Option<String> = None;
            loop {
                let response = client
                    .list_batch_operations(page_size, page_token.clone())
                    .await?;

                for operation in response.operations {
                    yield operation;
                }

                if let Some(next_page_token) = response.next_page_token {
                    page_token = Some(next_page_token);
                } else {
                    break;
                }
            }
        }
    }
}
