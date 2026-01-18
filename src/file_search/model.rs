//! Data models for File Search API
//!
//! This module contains the types used for file search operations,
//! including stores, documents, operations, and metadata.

use serde::{Deserialize, Serialize};
use strum::AsRefStr;
use time::OffsetDateTime;

use crate::common::serde::{
    deserialize_optional_string_to_i64, deserialize_string_to_i64, mime_as_string,
};

/// A file search store is a container for document embeddings.
///
/// Stores persist indefinitely until deleted, unlike raw files which expire after 48 hours.
/// You can create multiple stores to organize your documents.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSearchStore {
    /// Resource name (e.g., "fileSearchStores/my-store-123")
    pub name: String,

    /// Human-readable display name (max 512 chars)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Creation timestamp
    #[serde(with = "time::serde::rfc3339")]
    pub create_time: OffsetDateTime,

    /// Last update timestamp
    #[serde(with = "time::serde::rfc3339")]
    pub update_time: OffsetDateTime,

    /// Number of active documents
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_string_to_i64"
    )]
    pub active_documents_count: Option<i64>,

    /// Number of pending documents
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_string_to_i64"
    )]
    pub pending_documents_count: Option<i64>,

    /// Number of failed documents
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_string_to_i64"
    )]
    pub failed_documents_count: Option<i64>,

    /// Total size in bytes
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_string_to_i64"
    )]
    pub size_bytes: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFileSearchStoreRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

/// A document represents a file within a file search store.
///
/// Documents are automatically chunked, embedded, and indexed when uploaded.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    /// Resource name (e.g., "fileSearchStores/*/documents/doc-123")
    pub name: String,

    /// Human-readable display name (max 512 chars)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Custom metadata (max 20 items)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_metadata: Option<Vec<CustomMetadata>>,

    /// Last update timestamp
    #[serde(with = "time::serde::rfc3339")]
    pub update_time: OffsetDateTime,

    /// Creation timestamp
    #[serde(with = "time::serde::rfc3339")]
    pub create_time: OffsetDateTime,

    /// Current state
    pub state: DocumentState,

    /// Size in bytes
    #[serde(deserialize_with = "deserialize_string_to_i64")]
    pub size_bytes: i64,

    /// MIME type
    #[serde(with = "mime_as_string")]
    pub mime_type: mime::Mime,
}

/// The lifecycle state of a document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, AsRefStr)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum DocumentState {
    /// Unknown state
    StateUnspecified,
    /// Document chunks are being processed (embedding and indexing)
    StatePending,
    /// All chunks processed and ready for querying
    StateActive,
    /// Some chunks failed processing
    StateFailed,
}

/// Custom metadata for filtering and organizing documents.
///
/// Documents can have up to 20 metadata entries. Metadata can be used
/// to filter searches using AIP-160 syntax (e.g., `category = "api-docs"`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomMetadata {
    /// Metadata key
    pub key: String,
    /// Metadata value (string, string list, or numeric)
    #[serde(flatten)]
    pub value: CustomMetadataValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CustomMetadataValue {
    StringValue { string_value: String },
    StringListValue { string_list_value: StringList },
    NumericValue { numeric_value: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringList {
    pub values: Vec<String>,
}

/// Configuration for how documents are chunked.
///
/// Chunking splits documents into smaller pieces for more precise retrieval.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChunkingConfig {
    /// White space-based chunking configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub white_space_config: Option<WhiteSpaceConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhiteSpaceConfig {
    /// Maximum tokens per chunk
    pub max_tokens_per_chunk: u32,

    /// Maximum overlapping tokens between chunks
    pub max_overlap_tokens: u32,
}

/// A long-running operation for file uploads and imports.
///
/// Operations track the progress of file processing, including
/// chunking, embedding, and indexing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Operation {
    /// Operation name (e.g., "fileSearchStores/*/operations/*")
    pub name: String,

    /// Service-specific metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,

    /// Whether operation is complete
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub done: Option<bool>,

    /// Result (error or response)
    #[serde(flatten)]
    pub result: Option<OperationResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OperationResult {
    Error { error: Status },
    Response { response: serde_json::Value },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Status {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadToFileSearchStoreRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_metadata: Option<Vec<CustomMetadata>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunking_config: Option<ChunkingConfig>,

    #[serde(
        with = "mime_as_string::optional",
        skip_serializing_if = "Option::is_none"
    )]
    pub mime_type: Option<mime::Mime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportFileRequest {
    /// File resource name from Files API
    pub file_name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_metadata: Option<Vec<CustomMetadata>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunking_config: Option<ChunkingConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListFileSearchStoresResponse {
    pub file_search_stores: Vec<FileSearchStore>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListDocumentsResponse {
    pub documents: Vec<Document>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
}

pub fn extract_store_name(full_name: &str) -> Result<String, crate::client::Error> {
    // Extract store name from "fileSearchStores/{store}/documents/{doc}"
    let mut parts = full_name.split('/');
    if parts.next() == Some("fileSearchStores") {
        if let Some(store_id) = parts.next() {
            return Ok(format!("fileSearchStores/{}", store_id));
        }
    }
    Err(crate::client::Error::InvalidResourceName {
        name: full_name.to_string(),
    })
}

pub fn extract_document_id(full_name: &str) -> Result<String, crate::client::Error> {
    // Extract document ID from "fileSearchStores/{store}/documents/{doc}"
    let mut parts = full_name.split('/');
    if parts.next() == Some("fileSearchStores") {
        if let Some(_store) = parts.next() {
            if parts.next() == Some("documents") {
                if let Some(doc_id) = parts.next() {
                    return Ok(doc_id.to_string());
                }
            }
        }
    }
    Err(crate::client::Error::InvalidResourceName {
        name: full_name.to_string(),
    })
}
