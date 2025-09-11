use crate::{
    client::{Error as ClientError, GeminiClient},
    files::handle::GeminiFile,
};
use mime::Mime;
use std::sync::Arc;

/// A builder for creating a file resource.
pub struct FileBuilder {
    client: Arc<GeminiClient>,
    file_bytes: Vec<u8>,
    display_name: Option<String>,
    file_path: Option<String>,
    mime_type: Option<Mime>,
}

impl FileBuilder {
    pub(crate) fn new<B: Into<Vec<u8>>>(client: Arc<GeminiClient>, file_bytes: B) -> Self {
        Self {
            client,
            file_bytes: file_bytes.into(),
            display_name: None,
            file_path: None,
            mime_type: None,
        }
    }

    /// The display name of the file.
    pub fn display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    /// The path to the file to upload.
    pub fn from_path(mut self, file_path: impl Into<String>) -> Self {
        self.file_path = Some(file_path.into());
        self
    }

    /// The MIME type of the file.
    pub fn with_mime_type(mut self, mime_type: Mime) -> Self {
        self.mime_type = Some(mime_type);
        self
    }

    /// Upload the file.
    pub async fn upload(self) -> Result<GeminiFile, ClientError> {
        let mime_type = self.mime_type.unwrap_or(mime::APPLICATION_OCTET_STREAM);
        let file = self
            .client
            .upload_file(self.display_name, self.file_bytes, mime_type)
            .await?;

        Ok(GeminiFile::new(self.client, file))
    }
}
