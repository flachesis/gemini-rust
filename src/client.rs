use crate::{
    content_builder::ContentBuilder,
    embed_builder::EmbedBuilder,
    models::{
        BatchContentEmbeddingResponse, BatchEmbedContentsRequest, ContentEmbeddingResponse,
        EmbedContentRequest, GenerateContentRequest, GenerationResponse,
    },
    Error, Result,
};
use futures::stream::Stream;
use futures_util::StreamExt;
use reqwest::Client;
use serde_json::Value;
use std::pin::Pin;
use std::sync::Arc;
use url::Url;

const DEFAULT_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/";
const DEFAULT_MODEL: &str = "models/gemini-2.0-flash";

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

        let stream = response
            .bytes_stream()
            .map(|result| {
                match result {
                    Ok(bytes) => {
                        let text = String::from_utf8_lossy(&bytes);
                        // The stream returns each chunk as a separate JSON object
                        // Each line that starts with "data: " contains a JSON object
                        let mut responses = Vec::new();
                        for line in text.lines() {
                            if let Some(json_str) = line.strip_prefix("data: ") {
                                if json_str == "[DONE]" {
                                    continue;
                                }
                                match serde_json::from_str::<GenerationResponse>(json_str) {
                                    Ok(response) => responses.push(Ok(response)),
                                    Err(e) => responses.push(Err(Error::JsonError(e))),
                                }
                            }
                        }
                        futures::stream::iter(responses)
                    }
                    Err(e) => futures::stream::iter(vec![Err(Error::HttpError(e))]),
                }
            })
            .flatten();

        Ok(Box::pin(stream))
    }

    /// Embed content
    pub(crate) async fn embed_content(
        &self,
        request: EmbedContentRequest,
    ) -> Result<ContentEmbeddingResponse> {
        let value = self.embed(request, "embedContent").await?;
        let response = serde_json::from_value::<ContentEmbeddingResponse>(value)?;

        Ok(response)
    }

    /// Batch Embed content
    pub(crate) async fn embed_content_batch(
        &self,
        request: BatchEmbedContentsRequest,
    ) -> Result<BatchContentEmbeddingResponse> {
        let value = self.embed(request, "batchEmbedContents").await?;
        let response = serde_json::from_value::<BatchContentEmbeddingResponse>(value)?;

        Ok(response)
    }

    /// Embed content base function
    async fn embed<T: serde::Serialize>(&self, request: T, endpoint: &str) -> Result<Value> {
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
        // All Gemini API endpoints now use the format with colon:
        // "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key=$API_KEY"
        let url_str = format!(
            "{}{}:{}?key={}",
            self.base_url, self.model, endpoint, self.api_key
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
        Self::with_model(api_key, "models/gemini-2.0-pro-exp-02-05".to_string())
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
        let client = GeminiClient::with_base_url(api_key, model, base_url);
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
}
