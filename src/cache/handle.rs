use snafu::{ResultExt, Snafu};
use std::{result::Result, sync::Arc};

use crate::{
    cache::model::{CacheExpirationRequest, CachedContent, DeleteCachedContentResponse},
    client::{Error as ClientError, GeminiClient},
};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("client invocation error"))]
    Client { source: Box<ClientError> },

    #[snafu(display(
        "cache display name ('{display_name}') too long ({chars}), must be under 128 characters"
    ))]
    LongDisplayName { display_name: String, chars: usize },

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
