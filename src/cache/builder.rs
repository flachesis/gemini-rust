use snafu::ResultExt;
use std::{result::Result, sync::Arc, time::Duration};

use crate::{
    cache::handle::{CachedContentHandle, ClientSnafu, Error, LongDisplayNameSnafu},
    cache::model::{CacheExpirationRequest, CreateCachedContentRequest},
    client::GeminiClient,
    models::Content,
    tools::model::{Tool, ToolConfig},
};

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
    pub fn with_display_name<S: Into<String>>(mut self, display_name: S) -> Result<Self, Error> {
        let display_name = display_name.into();
        let chars = display_name.chars().count();
        snafu::ensure!(
            chars <= 128,
            LongDisplayNameSnafu {
                display_name,
                chars
            }
        );
        self.display_name = Some(display_name);
        Ok(self)
    }

    /// Set the system instruction for the cached content.
    pub fn with_system_instruction<S: Into<String>>(mut self, instruction: S) -> Self {
        self.system_instruction = Some(Content::text(instruction.into()));
        self
    }

    /// Add a user message to the cached content.
    pub fn with_user_message<S: Into<String>>(mut self, message: S) -> Self {
        self.contents
            .push(crate::models::Message::user(message.into()).content);
        self
    }

    /// Add a model message to the cached content.
    pub fn with_model_message<S: Into<String>>(mut self, message: S) -> Self {
        self.contents
            .push(crate::models::Message::model(message.into()).content);
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
