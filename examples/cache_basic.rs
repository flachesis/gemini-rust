use gemini_rust::{Gemini, Model};
use std::time::Duration;

// Include the story text at compile time
const GRIEF_EATER_STORY: &str = include_str!("../test_data/grief_eater.txt");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key =
        std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY environment variable must be set");

    let client = Gemini::with_model(api_key, Model::Gemini25FlashLite)?;

    println!("Creating cached content with the full story text...");

    // Create cached content with the full story for analysis
    let cache = client
        .create_cache()
        .with_display_name("Grief Eater Story Analysis Cache")?
        .with_system_instruction("You are a literary analyst specialized in horror and supernatural fiction. Analyze stories for themes, character development, narrative techniques, and psychological elements.")
        .with_user_message("Please read and analyze this story:")
        .with_user_message(GRIEF_EATER_STORY)
        .with_ttl(Duration::from_secs(3600)) // Cache for 1 hour
        .execute()
        .await?;

    println!("✅ Cache created successfully: {}", cache.name());

    // Demonstrate cache retrieval to show token count
    println!("\nRetrieving cache information...");
    let cached_content = cache.get().await?;
    println!("📋 Cache details:");
    println!("  - Name: {}", cached_content.name);
    println!(
        "  - Display Name: {}",
        cached_content
            .display_name
            .as_ref()
            .unwrap_or(&"N/A".to_string())
    );
    println!("  - Model: {}", cached_content.model);
    println!("  - Created: {}", cached_content.create_time);
    println!("  - Updated: {}", cached_content.update_time);
    println!(
        "  - Total tokens: {}",
        cached_content.usage_metadata.total_token_count
    );
    if let Some(expire_time) = cached_content.expiration.expire_time {
        println!("  - Expires: {}", expire_time);
    }

    // Ask several analytical questions using the cached content
    println!("\n=== Question 1: Main Theme ===");
    let response1 = client
        .generate_content()
        .with_cached_content(&cache)
        .with_user_message("What is the central theme of this story? How does the protagonist's relationship with grief evolve?")
        .execute()
        .await?;

    println!("🤖 {}", response1.text());

    println!("\n=== Question 2: Narrative Technique ===");
    let response2 = client
        .generate_content()
        .with_cached_content(&cache)
        .with_user_message("Analyze the narrative technique. How does the author use the protagonist's childhood nightmare as a literary device?")
        .execute()
        .await?;

    println!("🤖 {}", response2.text());

    println!("\n=== Question 3: Character Arc ===");
    let response3 = client
        .generate_content()
        .with_cached_content(&cache)
        .with_user_message("Describe the protagonist's character arc. What does he learn about grief by the end of the story?")
        .execute()
        .await?;

    println!("🤖 {}", response3.text());

    println!("\n=== Question 4: Symbolism ===");
    let response4 = client
        .generate_content()
        .with_cached_content(&cache)
        .with_user_message("What symbolic meaning does the grandfather's pocket knife hold in the story's resolution?")
        .execute()
        .await?;

    println!("🤖 {}", response4.text());

    // Clean up by deleting the cache
    println!("\nCleaning up cache...");
    match cache.delete().await {
        Ok(_) => println!("✅ Cache deleted successfully"),
        Err((cache, error)) => {
            println!("❌ Failed to delete cache: {}", error);
            println!(
                "Cache handle returned for potential retry: {}",
                cache.name()
            );
        }
    }

    Ok(())
}
