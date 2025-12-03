use gemini_rust::{prelude::*, ThinkingLevel};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");

    let gemini = Gemini::with_model(api_key, Model::Gemini3Pro)?;

    println!("=== Gemini 3 Pro: Thinking Level Demo ===\n");

    let response = gemini
        .generate_content()
        .with_user_message("Explain how quantum entanglement works in a way that a high school student could understand.")
        .with_thinking_level(ThinkingLevel::High)
        .with_thoughts_included(true)
        .execute()
        .await?;

    println!("Response with High Thinking Level:");
    println!("{}", response.text());

    if !response.thoughts().is_empty() {
        println!("\nThoughts:");
        for thought in response.thoughts() {
            println!("  - {thought}");
        }
    }

    if let Some(usage) = &response.usage_metadata {
        println!("\nUsage:");
        println!("  Prompt tokens: {:?}", usage.prompt_token_count);
        println!("  Response tokens: {:?}", usage.candidates_token_count);
        println!("  Thought tokens: {:?}", usage.thoughts_token_count);
        println!("  Total tokens: {:?}", usage.total_token_count);
    }

    println!("\n=== Gemini 3 Pro: Low Thinking Level Demo ===\n");

    let response_low = gemini
        .generate_content()
        .with_user_message("What is 2 + 2?")
        .with_thinking_level(ThinkingLevel::Low)
        .execute()
        .await?;

    println!("Response with Low Thinking Level:");
    println!("{}", response_low.text());

    if let Some(usage) = &response_low.usage_metadata {
        println!("\nUsage:");
        println!("  Prompt tokens: {:?}", usage.prompt_token_count);
        println!("  Response tokens: {:?}", usage.candidates_token_count);
        println!("  Thought tokens: {:?}", usage.thoughts_token_count);
        println!("  Total tokens: {:?}", usage.total_token_count);
    }

    println!("\n=== Gemini 3 Pro: Media Resolution Demo ===\n");
    println!("Note: This demo requires a base64-encoded image.");
    println!("To test media resolution, provide a base64-encoded image string.");
    println!("Example usage:");
    println!(r#"
    // Example with actual image:
    // let sample_image = std::fs::read("path/to/image.png")?;
    // let base64_image = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &sample_image);
    //
    // let response_media = gemini
    //     .generate_content()
    //     .with_user_message("Describe what you see in this image.")
    //     .with_inline_data_and_resolution(base64_image, "image/png", MediaResolutionLevel::MediaResolutionLow)
    //     .with_thinking_level(ThinkingLevel::Low)
    //     .execute()
    //     .await?;
    "#);

    Ok(())
}
