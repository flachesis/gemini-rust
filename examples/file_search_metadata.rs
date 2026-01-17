//! File search with custom metadata filtering example
//!
//! This example demonstrates how to:
//! - Upload files with custom metadata
//! - Filter searches using metadata
//! - Manage multiple documents in a store

use gemini_rust::prelude::*;
use gemini_rust::{CustomMetadata, CustomMetadataValue};
use mime::TEXT_PLAIN;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
    let client = Gemini::new(api_key)?;

    // Create file search store
    println!("Creating file search store...");
    let store = client
        .create_file_search_store()
        .with_display_name("Technical Documentation Store")
        .execute()
        .await?;

    println!("✓ Created store: {}", store.name());

    // Upload API documentation with metadata
    let api_docs = b"
    Authentication API:
    - Use Bearer tokens in the Authorization header
    - Tokens expire after 1 hour
    - Refresh tokens are valid for 30 days
    - Rate limit: 100 requests per minute
    ";

    println!("Uploading API documentation...");
    let mut op1 = store
        .upload(api_docs.to_vec())
        .with_display_name("Authentication Guide")
        .with_mime_type(TEXT_PLAIN)
        .with_custom_metadata(vec![CustomMetadata {
            key: "category".to_string(),
            value: CustomMetadataValue::StringValue {
                string_value: "api-docs".to_string(),
            },
        }])
        .execute()
        .await?;

    // Upload user guide with different metadata
    let user_guide = b"
    User Guide:
    - Getting started with the platform
    - Creating your first project
    - Inviting team members
    - Setting up notifications
    ";

    println!("Uploading user guide...");
    let mut op2 = store
        .upload(user_guide.to_vec())
        .with_display_name("User Guide")
        .with_mime_type(TEXT_PLAIN)
        .with_custom_metadata(vec![CustomMetadata {
            key: "category".to_string(),
            value: CustomMetadataValue::StringValue {
                string_value: "user-guide".to_string(),
            },
        }])
        .execute()
        .await?;

    // Wait for both uploads to complete
    println!("Waiting for processing...");
    op1.wait_until_done(Duration::from_secs(5), Some(Duration::from_secs(60)))
        .await?;
    op2.wait_until_done(Duration::from_secs(5), Some(Duration::from_secs(60)))
        .await?;

    println!("✓ All files uploaded and processed");

    // Query with metadata filter for API docs only
    println!("\nQuerying API docs only (with metadata filter)...");
    let response = client
        .generate_content()
        .with_user_message("How do I authenticate?")
        .with_tool(Tool::file_search(
            vec![store.name().to_string()],
            Some("category = \"api-docs\"".to_string()),
        ))
        .execute()
        .await?;

    println!("\n=== Response (API docs only) ===");
    println!("{}", response.text());

    // Query without filter (searches all documents)
    println!("\nQuerying all documents (no filter)...");
    let response2 = client
        .generate_content()
        .with_user_message("How do I get started?")
        .with_tool(Tool::file_search(vec![store.name().to_string()], None))
        .execute()
        .await?;

    println!("\n=== Response (all docs) ===");
    println!("{}", response2.text());

    // Clean up
    println!("\nCleaning up...");
    store.delete(true).await?;
    println!("✓ Store deleted");

    Ok(())
}
