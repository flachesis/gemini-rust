use mime::Mime;
use std::sync::Arc;
use tracing::instrument;

use crate::client::{Error, GeminiClient};
use crate::file_search::model::{ChunkingConfig, CustomMetadata};
use crate::file_search::OperationHandle;

/// Builder for uploading files directly to a file search store.
///
/// This uploads file data and processes it in one step. The file is chunked,
/// embedded, and indexed. A temporary file reference is created but deleted
/// after 48 hours, while the indexed data persists in the store.
///
/// # Example
///
/// ```no_run
/// use gemini_rust::prelude::*;
/// use mime::TEXT_PLAIN;
/// # async fn example(store: FileSearchStoreHandle) -> Result<(), Box<dyn std::error::Error>> {
/// let data = b"Document content here";
/// let mut operation = store
///     .upload(data.to_vec())
///     .with_display_name("My Document")
///     .with_mime_type(TEXT_PLAIN)
///     .execute()
///     .await?;
///
/// // Wait for processing
/// operation.wait_until_done(
///     std::time::Duration::from_secs(5),
///     Some(std::time::Duration::from_secs(60))
/// ).await?;
/// # Ok(())
/// # }
/// ```
pub struct UploadBuilder {
    pub(crate) client: Arc<GeminiClient>,
    pub(crate) store_name: String,
    pub(crate) file_data: Vec<u8>,
    pub(crate) display_name: Option<String>,
    pub(crate) mime_type: Option<Mime>,
    pub(crate) custom_metadata: Option<Vec<CustomMetadata>>,
    pub(crate) chunking_config: Option<ChunkingConfig>,
}

impl UploadBuilder {
    pub fn with_display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    pub fn with_mime_type(mut self, mime_type: Mime) -> Self {
        self.mime_type = Some(mime_type);
        self
    }

    pub fn with_custom_metadata(mut self, metadata: Vec<CustomMetadata>) -> Self {
        self.custom_metadata = Some(metadata);
        self
    }

    pub fn with_chunking_config(mut self, config: ChunkingConfig) -> Self {
        self.chunking_config = Some(config);
        self
    }

    #[instrument(skip_all, fields(
        store.name = %self.store_name,
        file.size = self.file_data.len(),
        display_name = self.display_name.as_deref(),
        mime.type = self.mime_type.as_ref().map(|m| m.to_string()),
        metadata.present = self.custom_metadata.is_some(),
        chunking.present = self.chunking_config.is_some(),
    ))]
    pub async fn execute(self) -> Result<OperationHandle, Error> {
        let operation = self
            .client
            .upload_to_file_search_store(
                &self.store_name,
                self.file_data,
                self.display_name,
                self.mime_type,
                self.custom_metadata,
                self.chunking_config,
            )
            .await?;

        Ok(OperationHandle::new(self.client, operation))
    }
}
