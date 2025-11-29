use std::sync::Arc;
use tracing::instrument;

use crate::client::{Error, GeminiClient};
use crate::file_search::model::{extract_document_id, extract_store_name, Document, DocumentState};

/// A handle for managing a document within a file search store.
///
/// Provides methods to check document state, refresh metadata, and delete documents.
#[derive(Debug, Clone)]
pub struct DocumentHandle {
    client: Arc<GeminiClient>,
    document: Document,
}

impl DocumentHandle {
    pub fn new(client: Arc<GeminiClient>, document: Document) -> Self {
        Self { client, document }
    }

    pub fn name(&self) -> &str {
        &self.document.name
    }

    pub fn state(&self) -> DocumentState {
        self.document.state
    }

    pub fn is_active(&self) -> bool {
        self.document.state == DocumentState::StateActive
    }

    pub fn document(&self) -> &Document {
        &self.document
    }

    #[instrument(skip_all, fields(document.name = %self.document.name))]
    pub async fn refresh(&mut self) -> Result<(), Error> {
        let store_name = extract_store_name(&self.document.name)?;
        let doc_id = extract_document_id(&self.document.name)?;
        self.document = self.client.get_document(&store_name, &doc_id).await?;
        Ok(())
    }

    #[instrument(skip_all, fields(document.name = %self.document.name, force))]
    pub async fn delete(self, force: bool) -> Result<(), Error> {
        let store_name = extract_store_name(&self.document.name)?;
        let doc_id = extract_document_id(&self.document.name)?;
        self.client
            .delete_document(&store_name, &doc_id, force)
            .await
    }
}
