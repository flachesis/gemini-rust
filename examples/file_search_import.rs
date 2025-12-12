//! File search with import from Files API example
//!
//! This example demonstrates how to:
//! - Upload a file via the Files API
//! - Import the file into a file search store
//! - Use the imported file for RAG

use gemini_rust::prelude::*;
use gemini_rust::{CustomMetadata, CustomMetadataValue};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
    let client = Gemini::new(api_key)?;

    // First, upload file via Files API
    let sample_text = b"
    The Gemini API provides several models:
    - Gemini 2.5 Flash: Fast and efficient for most tasks
    - Gemini 2.5 Pro: Advanced reasoning with thinking mode
    - Gemini 2.5 Flash Lite: Lightweight for simple tasks
    
    All models support multimodal input including text, images, and audio.
    The context window supports up to 2 million tokens.
    ";

    println!("Uploading file via Files API...");
    let file_handle = client
        .create_file(sample_text.to_vec())
        .with_mime_type(mime::TEXT_PLAIN)
        .display_name("Gemini Models Overview")
        .upload()
        .await?;

    println!("✓ File uploaded: {}", file_handle.name());

    // Files are typically ready immediately
    // Small wait to ensure processing is complete
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Create file search store
    println!("\nCreating file search store...");
    let store = client
        .create_file_search_store()
        .with_display_name("Imported Files Store")
        .execute()
        .await?;

    println!("✓ Created store: {}", store.name());

    // Import file into store
    println!("Importing file into store...");
    let mut operation = store
        .import_file(file_handle.name().to_string())
        .with_custom_metadata(vec![CustomMetadata {
            key: "source".to_string(),
            value: CustomMetadataValue::StringValue {
                string_value: "files-api".to_string(),
            },
        }])
        .execute()
        .await?;

    // Wait for import to complete
    println!("Waiting for import...");
    operation
        .wait_until_done(Duration::from_secs(5), Some(Duration::from_secs(60)))
        .await?;

    println!("✓ File imported successfully");

    // Use in generation
    println!("\nQuerying with file search...");
    let response = client
        .generate_content()
        .with_user_message("What models does Gemini API provide?")
        .with_tool(Tool::file_search(vec![store.name().to_string()], None))
        .execute()
        .await?;

    println!("\n=== Response ===");
    println!("{}", response.text());

    // Clean up
    println!("\nCleaning up...");
    store.delete(true).await?;
    if let Err((handle, _err)) = file_handle.delete().await {
        // File might already be cleaned up, that's okay
        eprintln!("Note: Could not delete file {}", handle.name());
    }
    println!("✓ Cleaned up");

    Ok(())
}
