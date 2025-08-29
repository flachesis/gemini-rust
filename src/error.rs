use thiserror::Error;

/// Errors that can occur when using the Gemini API
#[derive(Error, Debug)]
pub enum Error {
    /// Error from the reqwest HTTP client
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Error parsing JSON
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Error from the Gemini API
    #[error("Gemini API error: {status_code} - {message}")]
    ApiError {
        /// HTTP status code
        status_code: u16,
        /// Error message
        message: String,
    },

    /// Error building a valid request
    #[error("Request building error: {0}")]
    RequestError(String),

    /// Missing API key
    #[error("Missing API key")]
    MissingApiKey,

    /// Error with function calls
    #[error("Function call error: {0}")]
    FunctionCallError(String),

    /// Error converting between types
    #[error("Try from error: {0}")]
    TryFromError(String),

    /// Error indicating a batch operation has failed
    #[error("Batch operation failed: {name}")]
    BatchFailed { name: String, error: OperationError },

    /// Error indicating a batch operation has expired
    #[error("Batch operation expired: {name}")]
    BatchExpired { name: String },

    /// Error indicating an inconsistent state from the Batch API
    #[error("Inconsistent batch state: {description}")]
    InconsistentBatchState { description: String },

    /// Error indicating the batch operation resulted in an unsupported output format (e.g., GCS)
    #[error("Unsupported batch output: {description}")]
    UnsupportedBatchOutput { description: String },
}

/// Represents an error within a long-running operation.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct OperationError {
    pub code: i32,
    pub message: String,
    // details are not included as they are not consistently typed in the API
}

impl From<serde_json::Value> for Error {
    fn from(value: serde_json::Value) -> Self {
        Error::TryFromError(value.to_string())
    }
}
