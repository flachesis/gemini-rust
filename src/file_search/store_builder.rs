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
        let name = name.into();
        if name.len() > 512 {
            tracing::warn!(
                display_name.length = name.len(),
                "display_name exceeds 512 character limit"
            );
        }
        self.display_name = Some(name);
        self
    }

    #[instrument(skip_all, fields(
        display_name = self.display_name.as_deref(),
    ))]
    pub async fn execute(self) -> Result<FileSearchStoreHandle, Error> {
        let request = CreateFileSearchStoreRequest {
            display_name: self.display_name,
        };

        // Validate request before sending
        if let Err(err) = request.validate() {
            tracing::warn!(validation_error = %err, "invalid create store request");
            return Err(Error::FileSearchStorePrecondition { message: err });
        }

        let store = self.client.create_file_search_store(request).await?;

        Ok(FileSearchStoreHandle::new(self.client, store))
    }
}
