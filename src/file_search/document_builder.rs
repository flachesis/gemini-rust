use std::sync::Arc;

use async_stream::stream;
use futures::Stream;

use crate::client::{Error, GeminiClient};
use crate::file_search::DocumentHandle;

pub struct DocumentBuilder {
    pub(crate) client: Arc<GeminiClient>,
    pub(crate) store_name: String,
}

impl DocumentBuilder {
    pub async fn get(self, document_id: &str) -> Result<DocumentHandle, Error> {
        let document = self
            .client
            .get_document(&self.store_name, document_id)
            .await?;

        Ok(DocumentHandle::new(self.client, document))
    }

    pub async fn delete(self, document_id: &str, force: bool) -> Result<(), Error> {
        self.client
            .delete_document(&self.store_name, document_id, force)
            .await
    }

    pub fn list(self, page_size: Option<u32>) -> impl Stream<Item = Result<DocumentHandle, Error>> {
        stream! {
            let mut page_token: Option<String> = None;
            loop {
                let response = self
                    .client
                    .list_documents(&self.store_name, page_size, page_token.as_deref())
                    .await?;

                for document in response.documents {
                    yield Ok(DocumentHandle::new(self.client.clone(), document));
                }

                match response.next_page_token {
                    Some(token) => page_token = Some(token),
                    None => break,
                }
            }
        }
    }
}
