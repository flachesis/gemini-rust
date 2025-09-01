use crate::{
    batch_builder::BatchBuilder,
    content_builder::ContentBuilder,
    embed_builder::EmbedBuilder,
    models::{
        BatchContentEmbeddingResponse, BatchEmbedContentsRequest, BatchGenerateContentRequest,
        BatchGenerateContentResponse, BatchOperation, ContentEmbeddingResponse,
        EmbedContentRequest, GenerateContentRequest, GenerationResponse, ListBatchesResponse,
    },
    Batch,
};
use eventsource_stream::{EventStreamError, Eventsource};
use futures::{Stream, StreamExt, TryStreamExt};
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue, InvalidHeaderValue},
    Client, ClientBuilder, Response,
};
use serde::de::DeserializeOwned;
use snafu::{ResultExt, Snafu};
use std::{
    fmt::{self, Formatter},
    sync::{Arc, LazyLock},
};
use url::Url;

static DEFAULT_BASE_URL: LazyLock<Url> = LazyLock::new(|| {
    Url::parse("https://generativelanguage.googleapis.com/v1beta/")
        .expect("unreachable error: failed to parse default base URL")
});

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum Model {
    #[default]
    Gemini25Flash,
    Gemini25FlashLite,
    Gemini25Pro,
    TextEmbedding004,
    Custom(String),
}

impl Model {
    pub fn as_str(&self) -> &str {
        match self {
            Model::Gemini25Flash => "models/gemini-2.5-flash",
            Model::Gemini25FlashLite => "models/gemini-2.5-flash-lite",
            Model::Gemini25Pro => "models/gemini-2.5-pro",
            Model::TextEmbedding004 => "models/text-embedding-004",
            Model::Custom(model) => model,
        }
    }
}

impl From<String> for Model {
    fn from(model: String) -> Self {
        Self::Custom(model)
    }
}

impl fmt::Display for Model {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Model::Gemini25Flash => write!(f, "models/gemini-2.5-flash"),
            Model::Gemini25FlashLite => write!(f, "models/gemini-2.5-flash-lite"),
            Model::Gemini25Pro => write!(f, "models/gemini-2.5-pro"),
            Model::TextEmbedding004 => write!(f, "models/texte-mbedding-004"),
            Model::Custom(model) => write!(f, "{}", model),
        }
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("failed to parse API key"))]
    InvalidApiKey { source: InvalidHeaderValue },

    #[snafu(display("failed to construct URL (probably incorrect model name): {suffix}"))]
    ConstructUrl {
        source: url::ParseError,
        suffix: String,
    },

    #[snafu(display("failed to perform request to '{url}'"))]
    PerformRequest { source: reqwest::Error, url: Url },

    #[snafu(display(
        "bad response from server; code {code}; description: {}",
        description.as_deref().unwrap_or("none")
    ))]
    BadResponse {
        /// HTTP status code
        code: u16,
        /// HTTP error description
        description: Option<String>,
    },

    #[snafu(display("failed to obtain stream SSE part"))]
    BadPart {
        source: EventStreamError<reqwest::Error>,
    },

    #[snafu(display("failed to deserialize JSON response"))]
    Deserialize { source: serde_json::Error },

    #[snafu(display("failed to generate content"))]
    DecodeResponse { source: reqwest::Error },
}

/// Internal client for making requests to the Gemini API
pub(crate) struct GeminiClient {
    http_client: Client,
    pub model: Model,
    base_url: Url,
}

impl GeminiClient {
    /// Create a new client with custom base URL
    fn with_base_url<K: AsRef<str>, M: Into<Model>>(
        api_key: K,
        model: M,
        base_url: Url,
    ) -> Result<Self, Error> {
        let headers = HeaderMap::from_iter([(
            HeaderName::from_static("x-goog-api-key"),
            HeaderValue::from_str(api_key.as_ref()).context(InvalidApiKeySnafu)?,
        )]);

        let http_client = ClientBuilder::new()
            .default_headers(headers)
            .build()
            .expect("all parameters must be valid");

        Ok(Self {
            http_client,
            model: model.into(),
            base_url,
        })
    }

    /// Check the response status code and return an error if it is not successful
    async fn check_response(response: Response) -> Result<Response, Error> {
        let status = response.status();
        if !status.is_success() {
            let description = response.text().await.ok();
            BadResponseSnafu {
                code: status.as_u16(),
                description,
            }
            .fail()
        } else {
            Ok(response)
        }
    }

    /// Generate content
    pub(crate) async fn generate_content_raw(
        &self,
        request: GenerateContentRequest,
    ) -> Result<GenerationResponse, Error> {
        let url = self.build_url("generateContent")?;

        let response = self
            .http_client
            .post(url.clone())
            .json(&request)
            .send()
            .await
            .context(PerformRequestSnafu { url })?;

        Self::check_response(response)
            .await?
            .json()
            .await
            .context(DecodeResponseSnafu)
    }

    /// Generate content with streaming
    pub(crate) async fn generate_content_stream(
        &self,
        request: GenerateContentRequest,
    ) -> Result<impl TryStreamExt<Ok = GenerationResponse, Error = Error> + Send, Error> {
        let mut url = self.build_url("streamGenerateContent")?;
        url.query_pairs_mut().append_pair("alt", "sse");

        let response = self
            .http_client
            .post(url.clone())
            .json(&request)
            .send()
            .await
            .context(PerformRequestSnafu { url })?;

        Ok(Self::check_response(response)
            .await?
            .bytes_stream()
            .eventsource()
            .map(|event| event.context(BadPartSnafu))
            .map_ok(|event| {
                serde_json::from_str::<GenerationResponse>(&event.data).context(DeserializeSnafu)
            })
            .map(|r| r.flatten()))
    }

    /// Embed content
    pub(crate) async fn embed_content(
        &self,
        request: EmbedContentRequest,
    ) -> Result<ContentEmbeddingResponse, Error> {
        self.post_json(request, "embedContent").await
    }

    /// Batch Embed content
    pub(crate) async fn embed_content_batch(
        &self,
        request: BatchEmbedContentsRequest,
    ) -> Result<BatchContentEmbeddingResponse, Error> {
        self.post_json(request, "batchEmbedContents").await
    }

    /// Synchronous Batch Generate content
    pub(crate) async fn batch_generate_content_sync(
        &self,
        request: BatchGenerateContentRequest,
    ) -> Result<BatchGenerateContentResponse, Error> {
        let value = self.post_json(request, "batchGenerateContent").await?;
        serde_json::from_value(value).context(DeserializeSnafu)
    }

    /// Get a batch operation
    pub(crate) async fn get_batch_operation<T: serde::de::DeserializeOwned>(
        &self,
        name: &str,
    ) -> Result<T, Error> {
        let url = self.build_batch_url(name, None)?;

        let response = self
            .http_client
            .get(url.clone())
            .send()
            .await
            .context(PerformRequestSnafu { url })?;

        Self::check_response(response)
            .await?
            .json()
            .await
            .context(DecodeResponseSnafu)
    }

    /// List batch operations
    pub(crate) async fn list_batch_operations(
        &self,
        page_size: Option<u32>,
        page_token: Option<String>,
    ) -> Result<ListBatchesResponse, Error> {
        let mut url = self.build_batch_url("batches", None)?;

        if let Some(size) = page_size {
            url.query_pairs_mut()
                .append_pair("pageSize", &size.to_string());
        }
        if let Some(token) = page_token {
            url.query_pairs_mut().append_pair("pageToken", &token);
        }

        let response = self
            .http_client
            .get(url.clone())
            .send()
            .await
            .context(PerformRequestSnafu { url })?;

        Self::check_response(response)
            .await?
            .json()
            .await
            .context(DecodeResponseSnafu)
    }

    /// Cancel a batch operation
    pub(crate) async fn cancel_batch_operation(&self, name: &str) -> Result<(), Error> {
        let url = self.build_batch_url(name, Some("cancel"))?;
        let response = self
            .http_client
            .post(url.clone())
            .json(&serde_json::json!({}))
            .send()
            .await
            .context(PerformRequestSnafu { url })?;

        Self::check_response(response).await?;
        Ok(())
    }

    /// Delete a batch operation
    pub(crate) async fn delete_batch_operation(&self, name: &str) -> Result<(), Error> {
        let url = self.build_batch_url(name, None)?;
        let response = self
            .http_client
            .delete(url.clone())
            .send()
            .await
            .context(PerformRequestSnafu { url })?;

        Self::check_response(response).await?;
        Ok(())
    }

    /// Post JSON to an endpoint
    async fn post_json<I: serde::Serialize, O: DeserializeOwned>(
        &self,
        request: I,
        endpoint: &str,
    ) -> Result<O, Error> {
        let url = self.build_url(endpoint)?;

        let response = self
            .http_client
            .post(url.clone())
            .json(&request)
            .send()
            .await
            .context(PerformRequestSnafu { url })?;

        Self::check_response(response)
            .await?
            .json::<O>()
            .await
            .context(DecodeResponseSnafu)
    }

    /// Build a URL for the API
    fn build_url(&self, endpoint: &str) -> Result<Url, Error> {
        let url = self.base_url.clone();
        let suffix = format!("{}:{endpoint}", self.model);
        url.join(&suffix).context(ConstructUrlSnafu { suffix })
    }

    /// Build a URL for a batch operation
    fn build_batch_url(&self, name: &str, action: Option<&str>) -> Result<Url, Error> {
        let suffix = action
            .map(|a| format!("{name}:{a}"))
            .unwrap_or_else(|| name.to_string());

        let url = self.base_url.clone();
        url.join(&suffix).context(ConstructUrlSnafu { suffix })
    }
}

/// Client for the Gemini API
#[derive(Clone)]
pub struct Gemini {
    client: Arc<GeminiClient>,
}

impl Gemini {
    /// Create a new client with the specified API key
    pub fn new<K: AsRef<str>>(api_key: K) -> Result<Self, Error> {
        Self::with_model(api_key, Model::default())
    }

    /// Create a new client for the Gemini Pro model
    pub fn pro<K: AsRef<str>>(api_key: K) -> Result<Self, Error> {
        Self::with_model(api_key, Model::Gemini25Pro)
    }

    /// Create a new client with the specified API key and model
    pub fn with_model<K: AsRef<str>, M: Into<Model>>(api_key: K, model: M) -> Result<Self, Error> {
        Self::with_model_and_base_url(api_key, model, DEFAULT_BASE_URL.clone())
    }

    /// Create a new client with custom base URL
    pub fn with_base_url<K: AsRef<str>>(api_key: K, base_url: Url) -> Result<Self, Error> {
        Self::with_model_and_base_url(api_key, Model::default(), base_url)
    }

    /// Create a new client with the specified API key, model, and base URL
    pub fn with_model_and_base_url<K: AsRef<str>, M: Into<Model>>(
        api_key: K,
        model: M,
        base_url: Url,
    ) -> Result<Self, Error> {
        let client = GeminiClient::with_base_url(api_key, model.into(), base_url)?;
        Ok(Self {
            client: Arc::new(client),
        })
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
    ) -> impl Stream<Item = Result<BatchOperation, Error>> + Send {
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
