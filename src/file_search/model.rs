//! Data models for File Search API
//!
//! This module contains the types used for file search operations,
//! including stores, documents, operations, and metadata.

use serde::{Deserialize, Serialize};
use strum::AsRefStr;
use time::OffsetDateTime;

use crate::common::serde::i64_as_string;

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
        with = "i64_as_string::optional"
    )]
    pub active_documents_count: Option<i64>,

    /// Number of pending documents
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "i64_as_string::optional"
    )]
    pub pending_documents_count: Option<i64>,

    /// Number of failed documents
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "i64_as_string::optional"
    )]
    pub failed_documents_count: Option<i64>,

    /// Total size in bytes
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "i64_as_string::optional"
    )]
    pub size_bytes: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFileSearchStoreRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

impl CreateFileSearchStoreRequest {
    /// Validate the request according to API specifications
    pub fn validate(&self) -> Result<(), String> {
        if let Some(ref name) = self.display_name {
            if name.len() > 512 {
                return Err(format!(
                    "display_name length {} exceeds 512 character limit",
                    name.len()
                ));
            }
        }
        Ok(())
    }
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
    #[serde(with = "i64_as_string")]
    pub size_bytes: i64,

    /// MIME type
    pub mime_type: String,
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

impl PartialEq for CustomMetadataValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::StringValue { string_value: a }, Self::StringValue { string_value: b }) => {
                a == b
            }
            (
                Self::StringListValue {
                    string_list_value: a,
                },
                Self::StringListValue {
                    string_list_value: b,
                },
            ) => a == b,
            (Self::NumericValue { numeric_value: a }, Self::NumericValue { numeric_value: b }) => {
                // Handle NaN: treat NaN as equal to NaN for practical equality
                (a.is_nan() && b.is_nan()) || (a == b)
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

impl UploadToFileSearchStoreRequest {
    /// Validate the request according to API specifications
    pub fn validate(&self) -> Result<(), String> {
        if let Some(ref name) = self.display_name {
            if name.len() > 512 {
                return Err(format!(
                    "display_name length {} exceeds 512 character limit",
                    name.len()
                ));
            }
        }

        if let Some(ref metadata) = self.custom_metadata {
            if metadata.len() > 20 {
                return Err(format!(
                    "custom_metadata count {} exceeds 20 item limit",
                    metadata.len()
                ));
            }
        }

        Ok(())
    }
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

impl ImportFileRequest {
    /// Validate the request according to API specifications
    pub fn validate(&self) -> Result<(), String> {
        if let Some(ref metadata) = self.custom_metadata {
            if metadata.len() > 20 {
                return Err(format!(
                    "custom_metadata count {} exceeds 20 item limit",
                    metadata.len()
                ));
            }
        }

        Ok(())
    }
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

            /// Helper functions for file search validation and utility
            pub mod validation {
                use crate::client::Error;

                /// Validate document ID format according to API specifications
                #[allow(dead_code)]
                pub fn validate_document_id(id: &str) -> Result<(), Error> {
                    if id.is_empty() {
                        return Err(Error::InvalidResourceName {
                            name: "document_id cannot be empty".to_string(),
                        });
                    }

                    if id.len() > 40 {
                        return Err(Error::InvalidResourceName {
                            name: format!(
                                "document_id length {} exceeds 40 character limit",
                                id.len()
                            ),
                        });
                    }

                    // Check if all characters are lowercase alphanumeric or dashes
                    if !id
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
                    {
                        return Err(Error::InvalidResourceName {
                name: "document_id must contain only lowercase letters, numbers, and dashes".to_string(),
            });
                    }

                    // Check if it starts or ends with dash
                    if id.starts_with('-') || id.ends_with('-') {
                        return Err(Error::InvalidResourceName {
                            name: "document_id cannot start or end with dash".to_string(),
                        });
                    }

                    Ok(())
                }

                /// Validate store name format according to API specifications
                #[allow(dead_code)]
                pub fn validate_store_name(name: &str) -> Result<(), Error> {
                    if !name.starts_with("fileSearchStores/") {
                        return Err(Error::InvalidResourceName {
                            name: "store name must start with 'fileSearchStores/'".to_string(),
                        });
                    }

                    let store_id = &name["fileSearchStores/".len()..];
                    if store_id.is_empty() {
                        return Err(Error::InvalidResourceName {
                            name: "store ID cannot be empty".to_string(),
                        });
                    }

                    if store_id.len() > 40 {
                        return Err(Error::InvalidResourceName {
                            name: format!(
                                "store ID length {} exceeds 40 character limit",
                                store_id.len()
                            ),
                        });
                    }

                    // Check if all characters are lowercase alphanumeric or dashes
                    if !store_id
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
                    {
                        return Err(Error::InvalidResourceName {
                            name:
                                "store ID must contain only lowercase letters, numbers, and dashes"
                                    .to_string(),
                        });
                    }

                    // Check if it starts or ends with dash
                    if store_id.starts_with('-') || store_id.ends_with('-') {
                        return Err(Error::InvalidResourceName {
                            name: "store ID cannot start or end with dash".to_string(),
                        });
                    }

                    Ok(())
                }
            }
        }
    }
    Err(crate::client::Error::InvalidResourceName {
        name: full_name.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_metadata_value_equality_string() {
        let val1 = CustomMetadataValue::StringValue {
            string_value: "test".to_string(),
        };
        let val2 = CustomMetadataValue::StringValue {
            string_value: "test".to_string(),
        };
        let val3 = CustomMetadataValue::StringValue {
            string_value: "other".to_string(),
        };

        assert_eq!(val1, val2);
        assert_ne!(val1, val3);
    }

    #[test]
    fn test_custom_metadata_value_equality_string_list() {
        let val1 = CustomMetadataValue::StringListValue {
            string_list_value: StringList {
                values: vec!["a".to_string(), "b".to_string()],
            },
        };
        let val2 = CustomMetadataValue::StringListValue {
            string_list_value: StringList {
                values: vec!["a".to_string(), "b".to_string()],
            },
        };
        let val3 = CustomMetadataValue::StringListValue {
            string_list_value: StringList {
                values: vec!["c".to_string()],
            },
        };

        assert_eq!(val1, val2);
        assert_ne!(val1, val3);
    }

    #[test]
    fn test_custom_metadata_value_equality_numeric() {
        let val1 = CustomMetadataValue::NumericValue {
            numeric_value: 42.0,
        };
        let val2 = CustomMetadataValue::NumericValue {
            numeric_value: 42.0,
        };
        let val3 = CustomMetadataValue::NumericValue {
            numeric_value: 43.0,
        };

        assert_eq!(val1, val2);
        assert_ne!(val1, val3);
    }

    #[test]
    fn test_custom_metadata_value_equality_nan() {
        // Test that NaN == NaN for practical equality
        let val1 = CustomMetadataValue::NumericValue {
            numeric_value: f64::NAN,
        };
        let val2 = CustomMetadataValue::NumericValue {
            numeric_value: f64::NAN,
        };
        let val3 = CustomMetadataValue::NumericValue {
            numeric_value: 42.0,
        };

        assert_eq!(val1, val2, "NaN should equal NaN for CustomMetadataValue");
        assert_ne!(val1, val3, "NaN should not equal a regular number");
    }

    #[test]
    fn test_custom_metadata_value_equality_different_variants() {
        let string_val = CustomMetadataValue::StringValue {
            string_value: "42".to_string(),
        };
        let numeric_val = CustomMetadataValue::NumericValue {
            numeric_value: 42.0,
        };
        let list_val = CustomMetadataValue::StringListValue {
            string_list_value: StringList {
                values: vec!["42".to_string()],
            },
        };

        assert_ne!(string_val, numeric_val);
        assert_ne!(string_val, list_val);
        assert_ne!(numeric_val, list_val);
    }

    #[test]
    fn test_custom_metadata_equality() {
        let meta1 = CustomMetadata {
            key: "test".to_string(),
            value: CustomMetadataValue::StringValue {
                string_value: "value".to_string(),
            },
        };
        let meta2 = CustomMetadata {
            key: "test".to_string(),
            value: CustomMetadataValue::StringValue {
                string_value: "value".to_string(),
            },
        };
        let meta3 = CustomMetadata {
            key: "other".to_string(),
            value: CustomMetadataValue::StringValue {
                string_value: "value".to_string(),
            },
        };

        assert_eq!(meta1, meta2);
        assert_ne!(meta1, meta3);
    }
}
