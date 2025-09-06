//! The Cache module for managing cached content operations.
//!
//! This module provides the [`CachedContentHandle`] struct, which is a handle to a cached content
//! resource on the Gemini API. It allows for retrieving, updating, and deleting the cached content.
//!
//! The cache builder [`CacheBuilder`] provides a fluent API for creating cached content with
//! system instructions, conversation history, tools, and TTL configuration.
//!
//! ## Cache Usage
//!
//! Cached content allows you to cache large context (like system instructions and conversation
//! history) to reduce costs and improve performance for repeated API calls.
//!
//! ## Example usage:
//! ```rust,ignore
//! use gemini_rust::{Gemini, CacheExpiration};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Gemini::new(std::env::var("GEMINI_API_KEY")?)?;
//!     
//!     // Create a cache with system instruction and TTL
//!     let cache = client
//!         .create_cache()
//!         .with_model("models/gemini-2.5-flash")
//!         .with_system_instruction("You are a helpful assistant")
//!         .with_user_message("Hello, how are you?")
//!         .with_ttl(std::time::Duration::from_secs(3600)) // 1 hour
//!         .execute()
//!         .await?;
//!
//!     // Use the cached content in generation
//!     let response = client
//!         .generate_content()
//!         .with_cached_content(&cache)
//!         .with_user_message("What can you help me with?")
//!         .execute()
//!         .await?;
//!
//!     println!("Response: {}", response.text());
//!     Ok(())
//! }
//! ```

use snafu::{ResultExt, Snafu};
use std::{result::Result, sync::Arc, time::Duration};

use crate::{
    client::{Error as ClientError, GeminiClient},
    models::{
        CacheExpirationRequest, CachedContent, Content, CreateCachedContentRequest,
        DeleteCachedContentResponse,
    },
    tools::Tool,
    ToolConfig,
};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("client invocation error"))]
    Client { source: Box<ClientError> },

    #[snafu(display("expiration (TTL or expire time) is required for cache creation"))]
    MissingExpiration,
}

/// Represents a cached content resource, providing methods to manage its lifecycle.
///
/// A `CachedContentHandle` object is a handle to a cached content resource on the Gemini API.
/// It allows you to retrieve, update, or delete the cached content.
pub struct CachedContentHandle {
    /// The unique resource name of the cached content, e.g., `cachedContents/cache-xxxxxxxx`.
    pub name: String,
    client: Arc<GeminiClient>,
}

impl CachedContentHandle {
    /// Creates a new CachedContentHandle instance.
    pub(crate) fn new(name: String, client: Arc<GeminiClient>) -> Self {
        Self { name, client }
    }

    /// Returns the unique resource name of the cached content.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Retrieves the cached content configuration by making an API call.
    pub async fn get(&self) -> Result<CachedContent, Error> {
        self.client
            .get_cached_content(&self.name)
            .await
            .map_err(Box::new)
            .context(ClientSnafu)
    }

    /// Updates the cached content configuration (typically the TTL).
    pub async fn update(&self, expiration: CacheExpirationRequest) -> Result<CachedContent, Error> {
        self.client
            .update_cached_content(&self.name, expiration)
            .await
            .map_err(Box::new)
            .context(ClientSnafu)
    }

    /// Deletes the cached content resource from the server.
    pub async fn delete(self) -> Result<DeleteCachedContentResponse, (Self, ClientError)> {
        match self.client.delete_cached_content(&self.name).await {
            Ok(response) => Ok(response),
            Err(e) => Err((self, e)),
        }
    }
}

/// Builder for creating cached content with a fluent API.
pub struct CacheBuilder {
    client: Arc<GeminiClient>,
    display_name: Option<String>,
    contents: Vec<Content>,
    system_instruction: Option<Content>,
    tools: Vec<Tool>,
    tool_config: Option<ToolConfig>,
    expiration: Option<CacheExpirationRequest>,
}

impl CacheBuilder {
    /// Creates a new CacheBuilder instance.
    pub(crate) fn new(client: Arc<GeminiClient>) -> Self {
        Self {
            client,
            display_name: None,
            contents: Vec::new(),
            system_instruction: None,
            tools: Vec::new(),
            tool_config: None,
            expiration: None,
        }
    }

    /// Set a display name for the cached content.
    /// Maximum 128 Unicode characters.
    pub fn with_display_name<S: Into<String>>(mut self, display_name: S) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    /// Set the system instruction for the cached content.
    pub fn with_system_instruction<S: Into<String>>(mut self, instruction: S) -> Self {
        self.system_instruction = Some(Content::text(instruction.into()));
        self
    }

    /// Add a user message to the cached content.
    pub fn with_user_message<S: Into<String>>(mut self, message: S) -> Self {
        self.contents
            .push(crate::Message::user(message.into()).content);
        self
    }

    /// Add a model message to the cached content.
    pub fn with_model_message<S: Into<String>>(mut self, message: S) -> Self {
        self.contents
            .push(crate::Message::model(message.into()).content);
        self
    }

    /// Add content directly to the cached content.
    pub fn with_content(mut self, content: Content) -> Self {
        self.contents.push(content);
        self
    }

    /// Add multiple contents to the cached content.
    pub fn with_contents(mut self, contents: Vec<Content>) -> Self {
        self.contents.extend(contents);
        self
    }

    /// Add a tool to the cached content.
    pub fn with_tool(mut self, tool: Tool) -> Self {
        self.tools.push(tool);
        self
    }

    /// Add multiple tools to the cached content.
    pub fn with_tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools.extend(tools);
        self
    }

    /// Set the tool configuration.
    pub fn with_tool_config(mut self, tool_config: ToolConfig) -> Self {
        self.tool_config = Some(tool_config);
        self
    }

    /// Set the TTL (Time To Live) for the cached content.
    /// The cache will automatically expire after this duration.
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.expiration = Some(CacheExpirationRequest::from_ttl(ttl));
        self
    }

    /// Set an explicit expiration time for the cached content.
    pub fn with_expire_time(mut self, expire_time: time::OffsetDateTime) -> Self {
        self.expiration = Some(CacheExpirationRequest::from_expire_time(expire_time));
        self
    }

    /// Execute the cache creation request.
    pub async fn execute(self) -> Result<CachedContentHandle, Error> {
        let model = self.client.model.to_string();
        let expiration = self.expiration.ok_or(Error::MissingExpiration)?;

        let cached_content = CreateCachedContentRequest {
            display_name: self.display_name,
            model,
            contents: if self.contents.is_empty() {
                None
            } else {
                Some(self.contents)
            },
            tools: if self.tools.is_empty() {
                None
            } else {
                Some(self.tools)
            },
            system_instruction: self.system_instruction,
            tool_config: self.tool_config,
            expiration,
        };

        let response = self
            .client
            .create_cached_content(cached_content)
            .await
            .map_err(Box::new)
            .context(ClientSnafu)?;

        let cache_name = response.name;

        Ok(CachedContentHandle::new(cache_name, self.client))
    }
}
