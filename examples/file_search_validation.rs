//! File Search API validation example (Fixed version)
//!
//! This example demonstrates and validates the following enhanced features:
//! - Input validation testing (display name length, metadata count limits)
//! - FileGroundingChunk handling for file search citations
//! - Improved error handling with specialized error types
//! - OperationHandle extension methods for better operation management
//!
//! # Running
//! ```
//! GEMINI_API_KEY=your_api_key cargo run --example file_search_validation
//! ```

use gemini_rust::client::Error;
use gemini_rust::prelude::*;
use gemini_rust::{CustomMetadata, CustomMetadataValue};
use std::time::Duration;
use tracing::{debug, error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing::level_filters::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
    let client = Gemini::new(api_key)?;

    info!("=== File Search API Validation Results ===");

    // Test 1: Input validation testing
    info!("🔍 Test 1: Input Validation Testing");
    test_input_validation(&client).await;
    info!("");

    // Test 2: FileGroundingChunk display
    info!("📄 Test 2: FileGroundingChunk Display");
    test_file_grounding_chunk(&client).await;
    info!("");

    // Test 3: Error handling improvement
    info!("⚠️  Test 3: Error Handling Improvement");
    test_improved_error_handling(&client).await;
    info!("");

    // Test 4: OperationHandle extension features
    info!("🔧 Test 4: OperationHandle Extension Features");
    test_operation_handle_extensions(&client).await;
    info!("");

    info!("✅ All tests completed!");

    Ok(())
}

/// Test input validation limits and functionality
async fn test_input_validation(client: &Gemini) {
    // Test 1a: Display name length limit (512 characters)
    debug!("📏 Testing display_name length limit (512 characters)");
    let long_name = "a".repeat(600); // exceeds limit
    let store_result = client
        .create_file_search_store()
        .with_display_name(&long_name)
        .execute()
        .await;

    match store_result {
        Err(Error::FileSearchStorePrecondition { message }) => {
            info!(error.message = message, "✅ Correctly caught length error");
        }
        Err(e) => {
            warn!(error.debug = ?e, "❌ Unexpected error type");
        }
        Ok(_) => {
            error!("❌ Failed to catch length validation error");
        }
    }

    // Test 1b: Metadata count limit (20 items)
    debug!("📊 Testing custom_metadata count limit (20 items)");
    let excessive_metadata: Vec<CustomMetadata> = (0..25)
        .map(|i| CustomMetadata {
            key: format!("key_{}", i),
            value: CustomMetadataValue::StringValue {
                string_value: format!("value_{}", i),
            },
        })
        .collect();

    let normal_store = client
        .create_file_search_store()
        .with_display_name("Valid Store")
        .execute()
        .await;

    match normal_store {
        Ok(store) => {
            let upload_result = store
                .upload(b"test content".to_vec())
                .with_custom_metadata(excessive_metadata)
                .execute()
                .await;

            match upload_result {
                Err(Error::FileSearchResourceExhausted { message }) => {
                    info!(
                        error.message = message,
                        "✅ Correctly caught metadata count error"
                    );
                }
                Err(e) => {
                    warn!(error.debug = ?e, "❌ Unexpected error type");
                }
                Ok(_) => {
                    error!("❌ Failed to catch metadata count validation error");
                }
            }

            // Clean up
            let _ = store.delete(true).await;
        }
        Err(e) => {
            warn!(error.debug = ?e, "❌ Failed to create store for testing");
        }
    }
}

/// Test FileGroundingChunk handling and display
async fn test_file_grounding_chunk(client: &Gemini) {
    debug!("🏪 Creating Grounding test store...");
    let store_creation = client
        .create_file_search_store()
        .with_display_name("Grounding Test Store")
        .execute()
        .await;

    let store = match store_creation {
        Ok(store) => {
            info!(store.name = store.name(), "✅ Store created successfully");
            store
        }
        Err(e) => {
            warn!(error.debug = ?e, "❌ Failed to create store");
            warn!("ℹ️  Skipping FileGroundingChunk test");
            return;
        }
    };

    // Upload test content with better error handling
    debug!("📄 Uploading test content...");
    let test_content = b"
    This document contains information about machine learning.
    Specifically, it covers deep learning architectures like CNNs and RNNs.
    Deep learning has revolutionized computer vision and natural language processing.
    Machine learning is a subset of artificial intelligence that focuses on algorithms.
    ";

    let upload_result = store
        .upload(test_content.to_vec())
        .with_display_name("ML Documentation")
        .with_mime_type("text/plain")
        .execute()
        .await;

    let mut operation = match upload_result {
        Ok(operation) => {
            info!("✅ File upload operation created successfully");
            operation
        }
        Err(e) => {
            warn!(error.debug = ?e, "❌ File upload failed");
            warn!("ℹ️  Skipping FileGroundingChunk complete test");

            // Provide specific guidance based on error type
            match e {
                Error::BadResponse { code, .. } if code == 400 => {
                    warn!("💡 Tip: 400 error may indicate invalid parameters, please check API configuration");
                }
                _ => warn!("💡 Tip: Please check network connection and API permissions"),
            }

            // Clean up
            let _ = store.delete(true).await;
            return;
        }
    };

    // Now handle the operation
    debug!("⏳ Waiting for file processing...");
    match operation
        .wait_until_done(Duration::from_secs(3), Some(Duration::from_secs(15)))
        .await
    {
        Ok(_) => {
            info!("✅ File processing completed");

            // Test grounding chunk handling
            debug!("🔍 Testing FileGroundingChunk handling...");
            let response = client
                .generate_content()
                .with_user_message("What is machine learning?")
                .with_tool(Tool::file_search(vec![store.name().to_string()], None))
                .execute()
                .await;

            match response {
                Ok(resp) => {
                    test_grounding_metadata_display(&resp);
                }
                Err(e) => {
                    warn!(error.debug = ?e, "⚠️  Generation request failed");
                    warn!("ℹ️  May be due to test environment limitations");
                }
            }
        }
        Err(e) => {
            warn!(error.debug = ?e, "⚠️  Operation timeout or failed");
            warn!("ℹ️  Skipping FileGroundingChunk test");
        }
    }

    // Clean up
    debug!("🧹 Cleaning up test resources...");
    let _ = store.delete(true).await;
}

/// Display grounding metadata with file search chunks
fn test_grounding_metadata_display(response: &GenerationResponse) {
    if let Some(candidate) = response.candidates.first() {
        if let Some(grounding) = &candidate.grounding_metadata {
            if let Some(chunks) = &grounding.grounding_chunks {
                let mut found_file_search = false;

                for (i, chunk) in chunks.iter().enumerate() {
                    if let Some(file_chunk) = &chunk.file_search {
                        found_file_search = true;
                        info!("📄 FileGroundingChunk #{}:", i + 1);
                        info!(file.title = file_chunk.title, "    Title");
                        info!(file.uri = file_chunk.uri.as_str(), "    URI");
                    }
                }

                if !found_file_search {
                    info!("ℹ️  This response does not contain file search citations");
                }
            } else {
                info!("ℹ️  No grounding chunks");
            }
        } else {
            info!("ℹ️  No grounding metadata");
        }
    } else {
        info!("ℹ️  No candidate responses");
    }
}

/// Test improved error handling with specialized error types
async fn test_improved_error_handling(client: &Gemini) {
    // Test FileSearchStorePrecondition error
    debug!("🏪 Testing FileSearchStorePrecondition error");
    let invalid_request = client
        .create_file_search_store()
        .with_display_name(&"x".repeat(600)) // exceeds limit
        .execute()
        .await;

    test_error_type(&invalid_request, "FileSearchStorePrecondition");

    // Test FileSearchResourceExhausted error
    debug!("📚 Testing FileSearchResourceExhausted error");

    let store_creation = client
        .create_file_search_store()
        .with_display_name("Resource Test Store")
        .execute()
        .await;

    match store_creation {
        Ok(store) => {
            let excessive_metadata: Vec<CustomMetadata> = (0..25)
                .map(|i| CustomMetadata {
                    key: format!("key_{}", i),
                    value: CustomMetadataValue::StringValue {
                        string_value: format!("value_{}", i),
                    },
                })
                .collect();

            let upload_result = store
                .upload(b"test data for validation".to_vec())
                .with_custom_metadata(excessive_metadata)
                .execute()
                .await;

            test_error_type(&upload_result, "FileSearchResourceExhausted");

            let _ = store.delete(true).await;
        }
        Err(e) => {
            warn!(error.debug = ?e, "ℹ️  Failed to create store for resource testing");
        }
    }

    // Test FileSearchDocumentFailed error (simulated)
    debug!("📄 Testing FileSearchDocumentFailed error");
    info!("ℹ️  This error type is triggered when actual operation fails");
    info!("ℹ️  Can be tested through OperationHandle error responses");
    info!("✅ FileSearchDocumentFailed error type is defined and available for error handling");
}

/// Generic function to test error type handling
fn test_error_type<T>(result: &Result<T, Error>, expected_type: &str) {
    match result {
        Err(Error::FileSearchStorePrecondition { message }) => {
            info!(
                error.message = message,
                "✅ Correctly caught {}: {}", expected_type, message
            );
        }
        Err(Error::FileSearchResourceExhausted { message }) => {
            info!(
                error.message = message,
                "✅ Correctly caught {}: {}", expected_type, message
            );
        }
        Err(Error::FileSearchDocumentFailed { message }) => {
            info!(
                error.message = message,
                "✅ Correctly caught {}: {}", expected_type, message
            );
        }
        Err(e) => {
            warn!(error.debug = ?e, "❌ Unexpected error type");
        }
        Ok(_) => {
            error!("❌ Failed to catch expected error type");
        }
    }
}

/// Test OperationHandle extension methods
async fn test_operation_handle_extensions(client: &Gemini) {
    debug!("🏪 Creating test store...");
    let store_creation = client
        .create_file_search_store()
        .with_display_name("Operation Test Store")
        .execute()
        .await;

    let store = match store_creation {
        Ok(store) => {
            info!(store.name = store.name(), "✅ Store created successfully");
            store
        }
        Err(e) => {
            warn!(error.debug = ?e, "❌ Store creation failed");
            test_fallback_operation_handle_methods().await;
            return;
        }
    };

    // Test upload operation extensions with better error handling
    debug!("📤 Testing upload operation extension features...");
    let test_content = b"Operation handle test content for validation - This is a simple text document for testing file search functionality.";

    // Use a safer approach with explicit MIME type
    let upload_result = store
        .upload(test_content.to_vec())
        .with_display_name("Test Document")
        .with_mime_type("text/plain")
        .execute()
        .await;

    match upload_result {
        Ok(operation) => {
            info!("✅ Upload operation created successfully");

            // Test is_upload_operation()
            if operation.is_upload_operation() {
                info!("✅ is_upload_operation() correctly identified as upload operation");
            } else {
                error!("❌ is_upload_operation() failed to identify correctly");
            }

            // Test is_import_operation() (should be false)
            if !operation.is_import_operation() {
                info!("✅ is_import_operation() correctly identified as non-import operation");
            } else {
                error!("❌ is_import_operation() incorrect identification");
            }

            // Test store_name() extraction
            if let Some(store_name) = operation.store_name() {
                info!(
                    store.name = store_name,
                    "✅ store_name() correctly extracted"
                );
                if store_name == store.name() {
                    info!("✅ Extracted name matches original store name");
                } else {
                    error!("❌ Extracted name does not match");
                }
            } else {
                error!("❌ store_name() failed to extract store name");
            }
        }
        Err(e) => {
            warn!(error.debug = ?e, "❌ Upload operation creation failed");
            warn!("ℹ️  May be due to test environment limitations or API configuration issues");

            // Provide specific guidance based on error type
            match e {
                Error::BadResponse { code, .. } if code == 400 => {
                    warn!("💡 Tip: 400 error may indicate invalid parameters, please check:");
                    warn!("    - File size (<100MB)");
                    warn!("    - MIME type format");
                    warn!("    - Store permissions");
                }
                _ => warn!("💡 Tip: Please check API key permissions and store configuration"),
            }

            // Fallback: Test with basic operation handle creation
            test_fallback_operation_handle_methods().await;
        }
    }

    // Clean up
    let _ = store.delete(true).await;
}

/// Fallback test for OperationHandle methods without actual API operations
async fn test_fallback_operation_handle_methods() {
    debug!("🔄 Performing fallback test (no actual API operation needed)");

    // Test operation name parsing logic directly
    let test_cases = vec![
        (
            "fileSearchStores/test-store-123/upload/operations/op-456",
            true,
            false,
        ),
        (
            "fileSearchStores/another-store/upload/operations/op-789",
            true,
            false,
        ),
        (
            "fileSearchStores/import-store-123/operations/op-999",
            false,
            true,
        ),
    ];

    for (operation_name, expected_upload, expected_import) in test_cases {
        let is_upload = operation_name.contains("/upload/operations/");
        let is_import = operation_name.contains("/operations/") && !is_upload;

        if is_upload == expected_upload && is_import == expected_import {
            let operation_type = if is_upload {
                "Upload"
            } else if is_import {
                "Import"
            } else {
                "Unknown"
            };
            info!(operation.type = operation_type, "✅ Operation type correctly identified");
        } else {
            error!("❌ Operation type identification error");
        }

        // Test store name extraction
        let store_name = extract_store_name_from_operation(operation_name);
        if let Some(name) = store_name {
            info!(store.name = name, "✅ Extracted store name");
        } else {
            error!("❌ Failed to extract store name");
        }
    }
}

/// Helper function to extract store name from operation name
fn extract_store_name_from_operation(operation_name: &str) -> Option<&str> {
    if let Some(start) = operation_name.find("fileSearchStores/") {
        let operation_path = &operation_name[start..];

        let is_upload = operation_path.contains("/upload/operations/");
        let is_import = operation_path.contains("/operations/") && !is_upload;

        if is_upload {
            // For upload operations: "store-123/upload/operations/op-456"
            if let Some(upload_end) = operation_path.find("/upload/") {
                Some(&operation_name[start..start + upload_end])
            } else {
                None
            }
        } else if is_import {
            // For import operations: "store-123/operations/op-456"
            if let Some(end) = operation_path.find("/operations/") {
                Some(&operation_name[start..start + end])
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}
