use std::sync::Arc;
use tracing::instrument;

use crate::client::{Error, GeminiClient};
use crate::file_search::model::CreateFileSearchStoreRequest;
use crate::file_search::FileSearchStoreHandle;

/// Builder for creating a new file search store.
///
/// # Example
///
/// ```no_run
/// use gemini_rust::prelude::*;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Gemini::new("API_KEY")?;
///
/// let store = client
///     .create_file_search_store()
///     .with_display_name("Documentation Store")
///     .execute()
///     .await?;
/// # Ok(())
/// # }
/// ```
pub struct FileSearchStoreBuilder {
    pub(crate) client: Arc<GeminiClient>,
    pub(crate) display_name: Option<String>,
}

impl FileSearchStoreBuilder {
    pub fn with_display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    #[instrument(skip_all, fields(
        display_name = self.display_name.as_deref(),
    ))]
    pub async fn execute(self) -> Result<FileSearchStoreHandle, Error> {
        let request = CreateFileSearchStoreRequest {
            display_name: self.display_name,
        };

        let store = self.client.create_file_search_store(request).await?;

        Ok(FileSearchStoreHandle::new(self.client, store))
    }
}
