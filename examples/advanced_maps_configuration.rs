//! Advanced Google Maps Configuration Example
//!
//! This example demonstrates advanced configuration options for Google Maps grounding,
//! including manual tool configuration and widget support.

use gemini_rust::prelude::*;
use gemini_rust::{LatLng, RetrievalConfig, Tool, ToolConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the Gemini client
    let gemini = Gemini::pro(std::env::var("GEMINI_API_KEY")?)?;

    println!("🔧 Advanced Google Maps Configuration Example\n");

    // Example 1: Manual tool configuration
    manual_tool_configuration(&gemini).await?;

    println!("\n{}\n", "=".repeat(60));

    // Example 2: Widget-enabled Maps grounding
    widget_enabled_grounding(&gemini).await?;

    println!("\n{}\n", "=".repeat(60));

    // Example 3: Comparison with and without location context
    location_comparison(&gemini).await?;

    Ok(())
}

/// Example 1: Manual Google Maps configuration with location context
async fn manual_tool_configuration(gemini: &Gemini) -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Example 1: Manual Tool Configuration");

    let prompt = "Find me vegan restaurants within a 10-minute drive.";

    // Manually create the tool and configuration
    let tool = Tool::google_maps(None); // No widget
    let tool_config = ToolConfig {
        retrieval_config: Some(RetrievalConfig {
            lat_lng: Some(LatLng::new(40.7128, -74.0060)), // New York City
        }),
        function_calling_config: None,
    };

    let response = gemini
        .generate_content()
        .with_user_message(prompt)
        .with_tool(tool)
        .with_tool_config(tool_config)
        .execute()
        .await?;

    println!("Query: {prompt}\n");
    println!("Response: {}", response.text());

    display_grounding_info(&response);

    Ok(())
}

/// Example 2: Google Maps grounding with widget support
async fn widget_enabled_grounding(gemini: &Gemini) -> Result<(), Box<dyn std::error::Error>> {
    println!("🗺️  Example 2: Widget-Enabled Grounding");

    let prompt = "Plan a food tour of downtown Chicago. Include 3-4 different types of cuisine.";

    let response = gemini
        .generate_content()
        .with_user_message(prompt)
        .with_tool(Tool::google_maps(Some(true))) // With widget enabled
        .with_tool_config(ToolConfig {
            retrieval_config: Some(RetrievalConfig {
                lat_lng: Some(LatLng::new(41.8781, -87.6298)), // Chicago
            }),
            function_calling_config: None,
        })
        .execute()
        .await?;

    println!("Query: {prompt}\n");
    println!("Response: {}", response.text());

    display_grounding_info(&response);

    // Check for widget context token
    if let Some(candidate) = response.candidates.first() {
        if let Some(grounding_metadata) = &candidate.grounding_metadata {
            if let Some(widget_token) = &grounding_metadata.google_maps_widget_context_token {
                println!("\n🎯 Widget Context Token Available:");
                println!(
                    "  Token: {}...",
                    &widget_token[..50.min(widget_token.len())]
                );
                println!("  Use this token to render an interactive Google Maps widget:");
                println!(
                    "  <gmp-place-contextual context-token=\"{widget_token}\"></gmp-place-contextual>"
                );
            } else {
                println!("\nℹ️  No widget context token in this response.");
            }
        }
    }

    Ok(())
}

/// Example 3: Compare responses with and without location context
async fn location_comparison(gemini: &Gemini) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Example 3: Location Context Comparison");

    let prompt = "What are some popular brunch spots?";

    // Query without location context
    println!("❌ WITHOUT location context:");
    let response_no_location = gemini
        .generate_content()
        .with_user_message(prompt)
        .execute()
        .await?;

    println!("Response: {}", response_no_location.text());
    check_for_grounding(&response_no_location);

    println!("\n---\n");

    // Query with location context and Google Maps
    println!("✅ WITH location context (Seattle):");
    let response_with_location = gemini
        .generate_content()
        .with_user_message(prompt)
        .with_tool(Tool::google_maps(None)) // No widget
        .with_tool_config(ToolConfig {
            retrieval_config: Some(RetrievalConfig {
                lat_lng: Some(LatLng::new(47.6062, -122.3321)), // Seattle
            }),
            function_calling_config: None,
        })
        .execute()
        .await?;

    println!("Response: {}", response_with_location.text());
    check_for_grounding(&response_with_location);

    Ok(())
}

/// Helper function to display grounding information
fn display_grounding_info(response: &GenerationResponse) {
    if let Some(candidate) = response.candidates.first() {
        if let Some(grounding_metadata) = &candidate.grounding_metadata {
            if let Some(chunks) = &grounding_metadata.grounding_chunks {
                println!("\n📍 Grounding Sources:");
                for (i, chunk) in chunks.iter().enumerate() {
                    if let Some(maps) = &chunk.maps {
                        println!("  {}. {} - {}", i + 1, maps.title, maps.uri);
                    } else if let Some(web) = &chunk.web {
                        println!("  {}. {} - {}", i + 1, web.title, web.uri);
                    }
                }
            }

            if let Some(supports) = &grounding_metadata.grounding_supports {
                println!("\n🔗 Grounding Supports:");
                for support in supports {
                    println!(
                        "  • \"{}\" -> {:?}",
                        support.segment.text, support.grounding_chunk_indices
                    );
                }
            }

            if let Some(queries) = &grounding_metadata.web_search_queries {
                println!("\n🔍 Search Queries:");
                for query in queries {
                    println!("  • {query}");
                }
            }
        } else {
            println!("\nℹ️  No grounding metadata available.");
        }
    }
}

/// Helper function to check for grounding in responses
fn check_for_grounding(response: &GenerationResponse) {
    if let Some(candidate) = response.candidates.first() {
        if let Some(grounding_metadata) = &candidate.grounding_metadata {
            if let Some(chunks) = &grounding_metadata.grounding_chunks {
                println!("  ✅ Grounded with {} source(s)", chunks.len());
            } else {
                println!("  ❌ No grounding sources");
            }
        } else {
            println!("  ❌ No grounding metadata");
        }
    } else {
        println!("  ❌ No candidates in response");
    }
}
