//! Basic file search example
//!
//! This example demonstrates how to:
//! - Create a file search store
//! - Upload a file to the store
//! - Use the store for RAG (retrieval augmented generation)
//! - Query with grounding citations

use gemini_rust::prelude::*;
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
        .with_display_name("Example Documentation Store")
        .execute()
        .await?;

    println!("✓ Created store: {}", store.name());

    // Upload a sample text file
    let sample_text = br"
    Robert Graves was an English poet, novelist, and critic. 
    He was born on July 24, 1895, in Wimbledon, London.
    Graves is best known for his novel 'I, Claudius' (1934), 
    a historical fiction about the Roman emperor Claudius.
    He also wrote 'Goodbye to All That' (1929), an autobiography 
    of his experiences during World War I.
    Graves died on December 7, 1985, in Deia, Majorca, Spain.
    ";

    println!("Uploading file to store...");
    let mut operation = store
        .upload(sample_text.to_vec())
        .with_display_name("Robert Graves Biography")
        .with_mime_type(TEXT_PLAIN)
        .execute()
        .await?;

    // Wait for processing
    println!("Waiting for file processing...");
    operation
        .wait_until_done(Duration::from_secs(5), Some(Duration::from_secs(60)))
        .await?;

    println!("✓ File uploaded and processed");

    // Use file search in generation
    println!("\nQuerying with file search...");
    let response = client
        .generate_content()
        .with_user_message("When and where was Robert Graves born?")
        .with_tool(Tool::file_search(vec![store.name().to_string()], None))
        .execute()
        .await?;

    println!("\n=== Response ===");
    println!("{}", response.text());

    // Check for grounding metadata
    if let Some(candidate) = response.candidates.first() {
        if let Some(metadata) = &candidate.grounding_metadata {
            println!("\n=== Grounding Metadata ===");
            println!("{:#?}", metadata);
        }
    }

    // Clean up
    println!("\nCleaning up...");
    store.delete(true).await?;
    println!("✓ Store deleted");

    Ok(())
}
