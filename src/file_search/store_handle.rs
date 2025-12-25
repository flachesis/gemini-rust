use std::sync::Arc;
use tracing::instrument;

use crate::client::{Error, GeminiClient};
use crate::file_search::model::FileSearchStore;
use crate::file_search::{DocumentBuilder, ImportBuilder, UploadBuilder};

/// A handle for managing a file search store.
///
/// Provides methods to upload files, import files, manage documents,
/// and delete the store. The store persists indefinitely until explicitly deleted.
///
/// # Example
///
/// ```no_run
/// use gemini_rust::prelude::*;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Gemini::new("API_KEY")?;
///
/// // Create a store
/// let store = client
///     .create_file_search_store()
///     .with_display_name("My Store")
///     .execute()
///     .await?;
///
/// // Upload a file
/// let data = b"Sample document content";
/// let operation = store
///     .upload(data.to_vec())
///     .with_display_name("Sample Doc")
///     .execute()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct FileSearchStoreHandle {
    client: Arc<GeminiClient>,
    store: FileSearchStore,
}

impl FileSearchStoreHandle {
    pub fn new(client: Arc<GeminiClient>, store: FileSearchStore) -> Self {
        Self { client, store }
    }

    pub fn name(&self) -> &str {
        &self.store.name
    }

    pub fn display_name(&self) -> Option<&str> {
        self.store.display_name.as_deref()
    }

    pub fn active_documents_count(&self) -> Option<i64> {
        self.store.active_documents_count
    }

    pub fn pending_documents_count(&self) -> Option<i64> {
        self.store.pending_documents_count
    }

    pub fn failed_documents_count(&self) -> Option<i64> {
        self.store.failed_documents_count
    }

    pub fn size_bytes(&self) -> Option<i64> {
        self.store.size_bytes
    }

    pub fn store(&self) -> &FileSearchStore {
        &self.store
    }

    #[instrument(skip_all, fields(store.name = %self.store.name))]
    pub async fn refresh(&mut self) -> Result<(), Error> {
        self.store = self.client.get_file_search_store(&self.store.name).await?;
        Ok(())
    }

    #[instrument(skip_all, fields(store.name = %self.store.name, force))]
    pub async fn delete(self, force: bool) -> Result<(), Error> {
        self.client
            .delete_file_search_store(&self.store.name, force)
            .await
    }

    pub fn upload(&self, file_data: Vec<u8>) -> UploadBuilder {
        UploadBuilder {
            client: self.client.clone(),
            store_name: self.store.name.clone(),
            file_data,
            display_name: None,
            mime_type: None,
            custom_metadata: None,
            chunking_config: None,
        }
    }

    pub fn import_file(&self, file_name: String) -> ImportBuilder {
        ImportBuilder {
            client: self.client.clone(),
            store_name: self.store.name.clone(),
            file_name,
            custom_metadata: None,
            chunking_config: None,
        }
    }

    pub fn documents(&self) -> DocumentBuilder {
        DocumentBuilder {
            client: self.client.clone(),
            store_name: self.store.name.clone(),
        }
    }
}
