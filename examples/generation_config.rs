use gemini_rust::{Gemini, GenerationConfig};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key from environment variable
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY environment variable not set");

    // Create client
    let client = Gemini::new(api_key).expect("unable to create Gemini API client");

    // Using the full generation config
    println!("--- Using full generation config ---");
    let response1 = client
        .generate_content()
        .with_system_prompt("You are a helpful assistant.")
        .with_user_message("Write a short poem about Rust programming language.")
        .with_generation_config(GenerationConfig {
            temperature: Some(0.9),
            top_p: Some(0.8),
            top_k: Some(20),
            max_output_tokens: Some(200),
            candidate_count: Some(1),
            stop_sequences: Some(vec!["END".to_string()]),
            response_mime_type: None,
            response_schema: None,
            thinking_config: None,
            ..Default::default()
        })
        .execute()
        .await?;

    println!(
        "Response with high temperature (0.9):\n{}\n",
        response1.text()
    );

    // Using individual generation parameters
    println!("--- Using individual generation parameters ---");
    let response2 = client
        .generate_content()
        .with_system_prompt("You are a helpful assistant.")
        .with_user_message("Write a short poem about Rust programming language.")
        .with_temperature(0.2)
        .with_max_output_tokens(100)
        .execute()
        .await?;

    println!(
        "Response with low temperature (0.2):\n{}\n",
        response2.text()
    );

    // Setting multiple parameters individually
    println!("--- Setting multiple parameters individually ---");
    let response3 = client
        .generate_content()
        .with_system_prompt("You are a helpful assistant.")
        .with_user_message("List 3 benefits of using Rust.")
        .with_temperature(0.7)
        .with_top_p(0.9)
        .with_max_output_tokens(150)
        .with_stop_sequences(vec!["4.".to_string()])
        .execute()
        .await?;

    println!(
        "Response with custom parameters and stop sequence:\n{}",
        response3.text()
    );

    Ok(())
}
