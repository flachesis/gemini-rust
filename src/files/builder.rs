use mime::Mime;
use snafu::ResultExt;
use std::sync::Arc;
use tracing::{debug, instrument, Span};

use super::*;
use crate::client::GeminiClient;

/// A builder for creating a file resource.
pub struct FileBuilder {
    client: Arc<GeminiClient>,
    file_bytes: Vec<u8>,
    display_name: Option<String>,
    mime_type: Option<Mime>,
}

impl FileBuilder {
    pub(crate) fn new<B: Into<Vec<u8>>>(client: Arc<GeminiClient>, file_bytes: B) -> Self {
        Self {
            client,
            file_bytes: file_bytes.into(),
            display_name: None,
            mime_type: None,
        }
    }

    /// The display name of the file.
    pub fn display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    /// The MIME type of the file.
    pub fn with_mime_type(mut self, mime_type: Mime) -> Self {
        self.mime_type = Some(mime_type);
        self
    }

    /// Upload the file.
    #[instrument(skip_all, fields(file.size, mime.type, display.name))]
    pub async fn upload(self) -> Result<super::handle::FileHandle, super::Error> {
        let mime_type = self.mime_type.unwrap_or(mime::APPLICATION_OCTET_STREAM);

        Span::current()
            .record("file.size", self.file_bytes.len())
            .record("mime.type", mime_type.to_string())
            .record(
                "display.name",
                self.display_name.as_deref().unwrap_or("none"),
            );
        debug!("uploading file");

        let file = self
            .client
            .upload_file(self.display_name, self.file_bytes, mime_type)
            .await
            .context(ClientSnafu)?;

        Ok(super::handle::FileHandle::new(self.client, file))
    }
}
