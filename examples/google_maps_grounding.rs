//! Google Maps Grounding Examples
//!
//! This example demonstrates how to use the Google Maps grounding functionality
//! in the gemini-rust library to provide location-aware responses.

use gemini_rust::prelude::*;
use gemini_rust::{LatLng, RetrievalConfig, Tool, ToolConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the Gemini client
    let gemini = Gemini::pro(std::env::var("GEMINI_API_KEY")?)?;

    println!("=== Google Maps Grounding Examples ===\n");

    // Example 1: Basic location-aware restaurant recommendations
    basic_restaurant_recommendations(&gemini).await?;

    println!("\n{}\n", "=".repeat(50));

    // Example 2: Place-specific question with location context
    place_specific_question(&gemini).await?;

    println!("\n{}\n", "=".repeat(50));

    // Example 3: Family-friendly recommendations with location
    family_friendly_recommendations(&gemini).await?;

    println!("\n{}\n", "=".repeat(50));

    // Example 4: Travel itinerary planning with widget support
    travel_itinerary_planning(&gemini).await?;

    Ok(())
}

/// Example 1: Basic restaurant recommendations within walking distance
async fn basic_restaurant_recommendations(
    gemini: &Gemini,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🍽️  Example 1: Italian Restaurants Within Walking Distance");

    let prompt = "What are the best Italian restaurants within a 15-minute walk from here?";

    let response = gemini
        .generate_content()
        .with_user_message(prompt)
        .with_tool(Tool::google_maps(None)) // No widget
        .with_tool_config(ToolConfig {
            retrieval_config: Some(RetrievalConfig {
                lat_lng: Some(LatLng::new(34.050481, -118.248526)), // Los Angeles coordinates
            }),
            function_calling_config: None,
        })
        .execute()
        .await?;

    println!("Question: {prompt}\n");
    println!("Response: {}", response.text());

    // Display grounding sources if available
    if let Some(candidate) = response.candidates.first() {
        if let Some(grounding_metadata) = &candidate.grounding_metadata {
            if let Some(chunks) = &grounding_metadata.grounding_chunks {
                println!("\n📍 Sources:");
                for chunk in chunks {
                    if let Some(maps) = &chunk.maps {
                        println!("  • [{}]({})", maps.title, maps.uri);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Example 2: Specific place question with outdoor seating
async fn place_specific_question(gemini: &Gemini) -> Result<(), Box<dyn std::error::Error>> {
    println!("🪑 Example 2: Cafe with Outdoor Seating");

    let prompt = "Is there a cafe near the corner of 1st and Main that has outdoor seating?";

    let response = gemini
        .generate_content()
        .with_user_message(prompt)
        .with_tool(Tool::google_maps(None)) // No widget
        .with_tool_config(ToolConfig {
            retrieval_config: Some(RetrievalConfig {
                lat_lng: Some(LatLng::new(34.050481, -118.248526)), // Los Angeles coordinates
            }),
            function_calling_config: None,
        })
        .execute()
        .await?;

    println!("Question: {prompt}\n");
    println!("Response: {}", response.text());

    // Display grounding sources if available
    if let Some(candidate) = response.candidates.first() {
        if let Some(grounding_metadata) = &candidate.grounding_metadata {
            if let Some(chunks) = &grounding_metadata.grounding_chunks {
                println!("\n📍 Sources:");
                for chunk in chunks {
                    if let Some(maps) = &chunk.maps {
                        println!("  • [{}]({})", maps.title, maps.uri);
                        if let Some(place_id) = &maps.place_id {
                            println!("    Place ID: {place_id}");
                        }
                    }
                }
            }

            // Display web search queries if available
            if let Some(queries) = &grounding_metadata.web_search_queries {
                println!("\n🔍 Search Queries:");
                for query in queries {
                    println!("  • {query}");
                }
            }
        }
    }

    Ok(())
}

/// Example 3: Family-friendly restaurants with playground reviews
async fn family_friendly_recommendations(
    gemini: &Gemini,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("👨‍👩‍👧‍👦 Example 3: Family-Friendly Restaurants with Playgrounds");

    let prompt = "Which family-friendly restaurants near here have the best playground reviews?";

    let response = gemini
        .generate_content()
        .with_user_message(prompt)
        .with_tool(Tool::google_maps(None)) // No widget
        .with_tool_config(ToolConfig {
            retrieval_config: Some(RetrievalConfig {
                lat_lng: Some(LatLng::new(30.2672, -97.7431)), // Austin, TX coordinates
            }),
            function_calling_config: None,
        })
        .execute()
        .await?;

    println!("Question: {prompt}\n");
    println!("Response: {}", response.text());

    // Display grounding sources and supports
    if let Some(candidate) = response.candidates.first() {
        if let Some(grounding_metadata) = &candidate.grounding_metadata {
            if let Some(chunks) = &grounding_metadata.grounding_chunks {
                println!("\n📍 Sources:");
                for (i, chunk) in chunks.iter().enumerate() {
                    if let Some(maps) = &chunk.maps {
                        println!("  {}. [{}]({})", i + 1, maps.title, maps.uri);
                    }
                }
            }

            // Display grounding supports to see which parts of the response are grounded
            if let Some(supports) = &grounding_metadata.grounding_supports {
                println!("\n🔗 Grounded Segments:");
                for support in supports {
                    println!(
                        "  • \"{}\" -> Sources: {:?}",
                        support.segment.text, support.grounding_chunk_indices
                    );
                }
            }
        }
    }

    Ok(())
}

/// Example 4: Travel itinerary planning with widget support
async fn travel_itinerary_planning(gemini: &Gemini) -> Result<(), Box<dyn std::error::Error>> {
    println!("✈️  Example 4: San Francisco Day Trip Planning");

    let prompt = "Plan a day in San Francisco for me. I want to see the Golden Gate Bridge, visit a museum, and have a nice dinner.";

    let response = gemini
        .generate_content()
        .with_user_message(prompt)
        .with_tool(Tool::google_maps(Some(true))) // With widget enabled
        .with_tool_config(ToolConfig {
            retrieval_config: Some(RetrievalConfig {
                lat_lng: Some(LatLng::new(37.78193, -122.40476)), // San Francisco coordinates
            }),
            function_calling_config: None,
        })
        .execute()
        .await?;

    println!("Question: {prompt}\n");
    println!("Response: {}", response.text());

    // Display comprehensive grounding information
    if let Some(candidate) = response.candidates.first() {
        if let Some(grounding_metadata) = &candidate.grounding_metadata {
            if let Some(chunks) = &grounding_metadata.grounding_chunks {
                println!("\n📍 Sources:");
                for (i, chunk) in chunks.iter().enumerate() {
                    if let Some(maps) = &chunk.maps {
                        println!("  {}. [{}]({})", i + 1, maps.title, maps.uri);
                        if let Some(place_id) = &maps.place_id {
                            println!("     Place ID: {place_id}");
                        }
                    }
                }
            }

            // Display widget context token if available
            if let Some(widget_token) = &grounding_metadata.google_maps_widget_context_token {
                println!("\n🗺️  Google Maps Widget Context Token:");
                println!(
                    "  <gmp-place-contextual context-token=\"{widget_token}\"></gmp-place-contextual>"
                );
                println!("  \nYou can use this token to render an interactive Google Maps widget in your web application.");
            }

            // Display grounding supports with text segments
            if let Some(supports) = &grounding_metadata.grounding_supports {
                println!("\n🔗 Detailed Grounding Information:");
                for support in supports {
                    let segment = &support.segment;
                    println!("  • Text: \"{}\"", segment.text);
                    println!(
                        "    Location: chars {}-{}",
                        segment.start_index, segment.end_index
                    );
                    println!("    Sources: {:?}", support.grounding_chunk_indices);
                    println!();
                }
            }

            // Display search queries
            if let Some(queries) = &grounding_metadata.web_search_queries {
                println!("🔍 Search Queries Used:");
                for query in queries {
                    println!("  • {query}");
                }
            }
        }
    }

    Ok(())
}
