use gemini_rust::prelude::*;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key from environment variable
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY environment variable not set");
    // Using custom base URL
    let custom_base_url = "https://generativelanguage.googleapis.com/v1beta/";
    let client_custom = Gemini::with_model_and_base_url(
        api_key,
        "models/gemini-2.5-flash-lite-preview-06-17".to_string(),
        custom_base_url.to_string().parse().unwrap(),
    )
    .expect("unable to create Gemini API client");

    println!("Custom base URL client created successfully");

    let response = client_custom
        .generate_content()
        .with_system_prompt("You are a helpful assistant.")
        .with_user_message("Hello, can you tell me a joke about programming?")
        .with_generation_config(GenerationConfig {
            temperature: Some(0.7),
            max_output_tokens: Some(100),
            ..Default::default()
        })
        .execute()
        .await?;

    println!("Response: {}", response.text());

    Ok(())
}
