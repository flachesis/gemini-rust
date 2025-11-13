//! Simple Google Maps Grounding Example
//!
//! A minimal example showing how to use Google Maps grounding for location-aware queries.

use gemini_rust::prelude::*;
use gemini_rust::{LatLng, RetrievalConfig, Tool, ToolConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the Gemini client with API key from environment
    let gemini = Gemini::pro(std::env::var("GEMINI_API_KEY")?)?;

    println!("🗺️  Simple Google Maps Grounding Example\n");

    // Ask for restaurant recommendations with location context
    let prompt = "What are the best coffee shops near me?";

    println!("Query: {}", prompt);
    println!("Location: San Francisco (37.7749, -122.4194)\n");

    // Use Google Maps grounding with San Francisco coordinates
    let response = gemini
        .generate_content()
        .with_user_message(prompt)
        .with_tool(Tool::google_maps(None)) // No widget
        .with_tool_config(ToolConfig {
            retrieval_config: Some(RetrievalConfig {
                lat_lng: Some(LatLng::new(37.7749, -122.4194)), // San Francisco
            }),
            function_calling_config: None,
        })
        .execute()
        .await?;

    // Display the response
    println!("🤖 Response:");
    println!("{}", response.text());

    // Show grounding sources if available
    if let Some(candidate) = response.candidates.first() {
        if let Some(grounding_metadata) = &candidate.grounding_metadata {
            if let Some(chunks) = &grounding_metadata.grounding_chunks {
                println!("\n📍 Grounding Sources:");
                for chunk in chunks {
                    if let Some(maps) = &chunk.maps {
                        println!("  • {} - {}", maps.title, maps.uri);
                    }
                }
            } else {
                println!("\nℹ️  No grounding sources available for this response.");
            }
        } else {
            println!("\nℹ️  No grounding metadata available for this response.");
        }
    }

    Ok(())
}
