use std::sync::Arc;
use tracing::instrument;

use crate::client::{Error, GeminiClient};
use crate::file_search::model::{ChunkingConfig, CustomMetadata, ImportFileRequest};
use crate::file_search::OperationHandle;

pub struct ImportBuilder {
    pub(crate) client: Arc<GeminiClient>,
    pub(crate) store_name: String,
    pub(crate) file_name: String,
    pub(crate) custom_metadata: Option<Vec<CustomMetadata>>,
    pub(crate) chunking_config: Option<ChunkingConfig>,
}

impl ImportBuilder {
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
        file.name = %self.file_name,
        metadata.present = self.custom_metadata.is_some(),
        chunking.present = self.chunking_config.is_some(),
    ))]
    pub async fn execute(self) -> Result<OperationHandle, Error> {
        let request = ImportFileRequest {
            file_name: self.file_name,
            custom_metadata: self.custom_metadata,
            chunking_config: self.chunking_config,
        };

        let operation = self
            .client
            .import_file_to_search_store(&self.store_name, request)
            .await?;

        Ok(OperationHandle::new(self.client, operation))
    }
}
