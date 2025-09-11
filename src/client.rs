#[cfg(feature = "batch")]
use crate::batch::{
    builder::BatchBuilder,
    handle::Batch,
    model::{
        BatchGenerateContentRequest, BatchGenerateContentResponse, BatchOperation,
        ListBatchesResponse,
    },
};
#[cfg(feature = "cache")]
use crate::cache::{
    builder::CacheBuilder,
    handle::CachedContentHandle,
    model::{
        CacheExpirationRequest, CachedContent, CachedContentSummary, CreateCachedContentRequest,
        DeleteCachedContentResponse, ListCachedContentsResponse,
    },
};
#[cfg(feature = "embedding")]
use crate::embedding::{
    builder::EmbedBuilder,
    model::{
        BatchContentEmbeddingResponse, BatchEmbedContentsRequest, ContentEmbeddingResponse,
        EmbedContentRequest,
    },
};
#[cfg(feature = "files")]
use crate::files::{
    handle::GeminiFile,
    model::{File, ListFilesResponse},
};
#[cfg(feature = "generation")]
use crate::generation::{
    builder::ContentBuilder,
    model::{GenerateContentRequest, GenerationResponse},
};
use eventsource_stream::{EventStreamError, Eventsource};
use futures::{Stream, StreamExt, TryStreamExt};
use mime::Mime;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue, InvalidHeaderValue},
    Client, ClientBuilder, Response,
};
use serde::de::DeserializeOwned;
use serde_json::json;
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
#[snafu(visibility(pub))]
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

    #[snafu(display("failed to parse URL"))]
    UrlParse { source: url::ParseError },

    #[snafu(display("no download URI for file"))]
    #[cfg(feature = "files")]
    NoDownloadUri {
        meta: Box<crate::files::model::File>,
    },

    #[snafu(display("I/O error during file operations"))]
    Io { source: std::io::Error },
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
    #[cfg(feature = "generation")]
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
    #[cfg(feature = "generation")]
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
            .map(|r| match r {
                Ok(res) => res,
                Err(e) => Err(e),
            }))
    }

    /// Embed content
    #[cfg(feature = "embedding")]
    pub(crate) async fn embed_content(
        &self,
        request: EmbedContentRequest,
    ) -> Result<ContentEmbeddingResponse, Error> {
        self.post_json(request, "embedContent").await
    }

    /// Batch Embed content
    #[cfg(feature = "embedding")]
    pub(crate) async fn embed_content_batch(
        &self,
        request: BatchEmbedContentsRequest,
    ) -> Result<BatchContentEmbeddingResponse, Error> {
        self.post_json(request, "batchEmbedContents").await
    }

    /// Synchronous Batch Generate content
    #[cfg(feature = "batch")]
    pub(crate) async fn batch_generate_content_sync(
        &self,
        request: BatchGenerateContentRequest,
    ) -> Result<BatchGenerateContentResponse, Error> {
        let value = self.post_json(request, "batchGenerateContent").await?;
        serde_json::from_value(value).context(DeserializeSnafu)
    }

    /// Get a batch operation
    #[cfg(feature = "batch")]
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
    #[cfg(feature = "batch")]
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

    /// List files
    #[cfg(feature = "files")]
    pub(crate) async fn list_files(
        &self,
        page_size: Option<u32>,
        page_token: Option<String>,
    ) -> Result<ListFilesResponse, Error> {
        let mut url = self.build_files_url(None)?;

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
    #[cfg(feature = "batch")]
    pub(crate) async fn cancel_batch_operation(&self, name: &str) -> Result<(), Error> {
        let url = self.build_batch_url(name, Some("cancel"))?;
        let response = self
            .http_client
            .post(url.clone())
            .json(&json!({}))
            .send()
            .await
            .context(PerformRequestSnafu { url })?;

        Self::check_response(response).await?;
        Ok(())
    }

    /// Delete a batch operation
    #[cfg(feature = "batch")]
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

    /// Upload a file using the resumable upload protocol.
    #[cfg(feature = "files")]
    pub(crate) async fn upload_file(
        &self,
        display_name: Option<String>,
        file_bytes: Vec<u8>,
        mime_type: Mime,
    ) -> Result<File, Error> {
        // Step 1: Initiate resumable upload
        // The upload URL is different from the metadata URL, so we construct it relative to the base URL's root.
        let initiate_url =
            self.base_url
                .join("/upload/v1beta/files")
                .context(ConstructUrlSnafu {
                    suffix: "/upload/v1beta/files".to_string(),
                })?;

        let response = self
            .http_client
            .post(initiate_url.clone())
            .header("X-Goog-Upload-Protocol", "resumable")
            .header("X-Goog-Upload-Command", "start")
            .header(
                "X-Goog-Upload-Header-Content-Length",
                file_bytes.len().to_string(),
            )
            .header("X-Goog-Upload-Header-Content-Type", mime_type.to_string())
            .json(&json!({"file": {"displayName": display_name}}))
            .send()
            .await
            .context(PerformRequestSnafu {
                url: initiate_url.clone(),
            })?;

        let checked_response = Self::check_response(response).await?;

        let upload_url = checked_response
            .headers()
            .get("X-Goog-Upload-URL")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| Error::BadResponse {
                code: 500,
                description: Some("Missing upload URL in response".to_string()),
            })?;

        // Step 2: Upload file content
        let upload_response = self
            .http_client
            .post(upload_url)
            .header("X-Goog-Upload-Command", "upload, finalize")
            .header("X-Goog-Upload-Offset", "0")
            .body(file_bytes)
            .send()
            .await
            .map_err(|e| Error::PerformRequest {
                source: e,
                url: Url::parse(upload_url).unwrap_or_else(|_| initiate_url.clone()),
            })?;

        let final_response = Self::check_response(upload_response).await?;

        #[derive(serde::Deserialize)]
        struct UploadResponse {
            file: File,
        }

        let upload_response: UploadResponse =
            final_response.json().await.context(DecodeResponseSnafu)?;
        Ok(upload_response.file)
    }

    /// Get a file resource
    #[cfg(feature = "files")]
    pub(crate) async fn get_file(&self, name: &str) -> Result<File, Error> {
        let url = self.build_files_url(Some(name))?;
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

    /// Delete a file resource
    #[cfg(feature = "files")]
    pub(crate) async fn delete_file(&self, name: &str) -> Result<(), Error> {
        let url = self.build_files_url(Some(name))?;
        let response = self
            .http_client
            .delete(url.clone())
            .send()
            .await
            .context(PerformRequestSnafu { url })?;

        Self::check_response(response).await?;
        Ok(())
    }

    #[cfg(feature = "files")]
    pub(crate) async fn download_file(&self, name: &str) -> Result<Vec<u8>, Error> {
        let mut url = self
            .base_url
            .join(&format!("/download/v1beta/{name}:download"))
            .context(ConstructUrlSnafu {
                suffix: format!("/download/v1beta/{name}:download"),
            })?;
        url.query_pairs_mut().append_pair("alt", "media");

        let response = self
            .http_client
            .get(url.clone())
            .send()
            .await
            .context(PerformRequestSnafu { url: url.clone() })?;

        Self::check_response(response)
            .await?
            .bytes()
            .await
            .context(PerformRequestSnafu { url })
            .map(|b| b.to_vec())
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

    /// Create cached content
    #[cfg(feature = "cache")]
    pub(crate) async fn create_cached_content(
        &self,
        cached_content: CreateCachedContentRequest,
    ) -> Result<CachedContent, Error> {
        let url = self.build_cache_url(None)?;
        let response = self
            .http_client
            .post(url.clone())
            .json(&cached_content)
            .send()
            .await
            .context(PerformRequestSnafu { url })?;

        Self::check_response(response)
            .await?
            .json::<CachedContent>()
            .await
            .context(DecodeResponseSnafu)
    }

    /// Get cached content
    #[cfg(feature = "cache")]
    pub(crate) async fn get_cached_content(&self, name: &str) -> Result<CachedContent, Error> {
        let url = self.build_cache_url(Some(name))?;
        let response = self
            .http_client
            .get(url.clone())
            .send()
            .await
            .context(PerformRequestSnafu { url })?;

        Self::check_response(response)
            .await?
            .json::<CachedContent>()
            .await
            .context(DecodeResponseSnafu)
    }

    /// Update cached content (typically to update TTL)
    #[cfg(feature = "cache")]
    pub(crate) async fn update_cached_content(
        &self,
        name: &str,
        expiration: CacheExpirationRequest,
    ) -> Result<CachedContent, Error> {
        let url = self.build_cache_url(Some(name))?;

        // Create a minimal update payload with just the expiration
        let update_payload = match expiration {
            CacheExpirationRequest::Ttl { ttl } => json!({ "ttl": ttl }),
            CacheExpirationRequest::ExpireTime { expire_time } => {
                json!({ "expireTime": expire_time.format(&time::format_description::well_known::Rfc3339).unwrap() })
            }
        };

        let response = self
            .http_client
            .patch(url.clone())
            .json(&update_payload)
            .send()
            .await
            .context(PerformRequestSnafu { url })?;

        Self::check_response(response)
            .await?
            .json::<CachedContent>()
            .await
            .context(DecodeResponseSnafu)
    }

    /// Delete cached content
    #[cfg(feature = "cache")]
    pub(crate) async fn delete_cached_content(
        &self,
        name: &str,
    ) -> Result<DeleteCachedContentResponse, Error> {
        let url = self.build_cache_url(Some(name))?;
        let response = self
            .http_client
            .delete(url.clone())
            .send()
            .await
            .context(PerformRequestSnafu { url })?;

        // For DELETE operations, we might get an empty response, so handle that case
        if response.status().is_success() {
            Ok(DeleteCachedContentResponse {
                success: Some(true),
            })
        } else {
            Self::check_response(response)
                .await?
                .json::<DeleteCachedContentResponse>()
                .await
                .context(DecodeResponseSnafu)
        }
    }

    /// List cached contents
    #[cfg(feature = "cache")]
    pub(crate) async fn list_cached_contents(
        &self,
        page_size: Option<i32>,
        page_token: Option<String>,
    ) -> Result<ListCachedContentsResponse, Error> {
        let mut url = self.build_cache_url(None)?;

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
            .json::<ListCachedContentsResponse>()
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

    /// Build a URL for file operations
    fn build_files_url(&self, name: Option<&str>) -> Result<Url, Error> {
        let suffix = name
            .map(|n| format!("files/{}", n.strip_prefix("files/").unwrap_or(n)))
            .unwrap_or_else(|| "files".to_string());

        self.base_url
            .join(&suffix)
            .context(ConstructUrlSnafu { suffix })
    }

    /// Build a URL for cache operations
    fn build_cache_url(&self, name: Option<&str>) -> Result<Url, Error> {
        let suffix = name
            .map(|n| {
                if n.starts_with("cachedContents/") {
                    n.to_string()
                } else {
                    format!("cachedContents/{}", n)
                }
            })
            .unwrap_or_else(|| "cachedContents".to_string());

        self.base_url
            .join(&suffix)
            .context(ConstructUrlSnafu { suffix })
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
    #[cfg(feature = "generation")]
    pub fn generate_content(&self) -> ContentBuilder {
        ContentBuilder::new(self.client.clone())
    }

    /// Start building a content generation request
    #[cfg(feature = "embedding")]
    pub fn embed_content(&self) -> EmbedBuilder {
        EmbedBuilder::new(self.client.clone())
    }

    /// Start building a synchronous batch content generation request
    #[cfg(feature = "batch")]
    pub fn batch_generate_content_sync(&self) -> BatchBuilder {
        BatchBuilder::new(self.client.clone())
    }

    /// Get a handle to a batch operation by its name.
    #[cfg(feature = "batch")]
    pub fn get_batch(&self, name: &str) -> Batch {
        Batch::new(name.to_string(), self.client.clone())
    }

    /// Lists batch operations.
    ///
    /// This method returns a stream that handles pagination automatically.
    #[cfg(feature = "batch")]
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

    /// Create cached content with a fluent API.
    #[cfg(feature = "cache")]
    pub fn create_cache(&self) -> CacheBuilder {
        CacheBuilder::new(self.client.clone())
    }

    /// Get a handle to cached content by its name.
    #[cfg(feature = "cache")]
    pub fn get_cached_content(&self, name: &str) -> CachedContentHandle {
        CachedContentHandle::new(name.to_string(), self.client.clone())
    }

    /// Lists cached contents.
    ///
    /// This method returns a stream that handles pagination automatically.
    #[cfg(feature = "cache")]
    pub fn list_cached_contents(
        &self,
        page_size: impl Into<Option<i32>>,
    ) -> impl Stream<Item = Result<CachedContentSummary, Error>> + Send {
        let client = self.client.clone();
        let page_size = page_size.into();
        async_stream::try_stream! {
            let mut page_token: Option<String> = None;
            loop {
                let response = client
                    .list_cached_contents(page_size, page_token.clone())
                    .await?;

                for cached_content in response.cached_contents {
                    yield cached_content;
                }

                if let Some(next_page_token) = response.next_page_token {
                    page_token = Some(next_page_token);
                } else {
                    break;
                }
            }
        }
    }

    /// Start building a file resource
    #[cfg(feature = "files")]
    pub fn create_file<B: Into<Vec<u8>>>(&self, bytes: B) -> crate::files::builder::FileBuilder {
        crate::files::builder::FileBuilder::new(self.client.clone(), bytes)
    }

    /// Get a handle to a file by its name.
    #[cfg(feature = "files")]
    pub async fn get_file(&self, name: &str) -> Result<GeminiFile, Error> {
        let file = self.client.get_file(name).await?;
        Ok(GeminiFile::new(self.client.clone(), file))
    }

    /// Lists files.
    ///
    /// This method returns a stream that handles pagination automatically.
    #[cfg(feature = "files")]
    pub fn list_files(
        &self,
        page_size: impl Into<Option<u32>>,
    ) -> impl Stream<Item = Result<GeminiFile, Error>> + Send {
        let client = self.client.clone();
        let page_size = page_size.into();
        async_stream::try_stream! {
            let mut page_token: Option<String> = None;
            loop {
                let response = client
                    .list_files(page_size, page_token.clone())
                    .await?;

                for file in response.files {
                    yield GeminiFile::new(client.clone(), file);
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
