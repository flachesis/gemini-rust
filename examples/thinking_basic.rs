use gemini_rust::{Gemini, GenerationConfig, ThinkingConfig};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key from environment variable
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY environment variable not set");

    // Create client
    let client = Gemini::pro(api_key).expect("unable to create Gemini API client");

    println!("=== Gemini 2.5 Thinking Basic Example ===\n");

    // Example 1: Using default dynamic thinking
    println!(
        "--- Example 1: Dynamic thinking (model automatically determines thinking budget) ---"
    );
    let response1 = client
        .generate_content()
        .with_system_prompt("You are a helpful mathematics assistant.")
        .with_user_message(
            "Explain Occam's razor principle and provide a simple example from daily life.",
        )
        .with_dynamic_thinking()
        .with_thoughts_included(true)
        .execute()
        .await?;

    // Display thinking process
    let thoughts = response1.thoughts();
    if !thoughts.is_empty() {
        println!("Thinking summary:");
        for (i, thought) in thoughts.iter().enumerate() {
            println!("Thought {}: {}\n", i + 1, thought);
        }
    }

    println!("Answer: {}\n", response1.text());

    // Display token usage
    if let Some(usage) = &response1.usage_metadata {
        println!("Token usage:");
        println!("  Prompt tokens: {}", usage.prompt_token_count);
        println!("  Response tokens: {}", usage.candidates_token_count);
        if let Some(thinking_tokens) = usage.thoughts_token_count {
            println!("  Thinking tokens: {}", thinking_tokens);
        }
        println!("  Total tokens: {}\n", usage.total_token_count);
    }

    // Example 2: Set specific thinking budget
    println!("--- Example 2: Set thinking budget (1024 tokens) ---");
    let response2 = client
        .generate_content()
        .with_system_prompt("You are a helpful programming assistant.")
        .with_user_message("List 3 main advantages of using the Rust programming language")
        .with_thinking_budget(1024)
        .with_thoughts_included(true)
        .execute()
        .await?;

    // Display thinking process
    let thoughts2 = response2.thoughts();
    if !thoughts2.is_empty() {
        println!("Thinking summary:");
        for (i, thought) in thoughts2.iter().enumerate() {
            println!("Thought {}: {}\n", i + 1, thought);
        }
    }

    println!("Answer: {}\n", response2.text());

    // Example 3: Disable thinking feature
    println!("--- Example 3: Disable thinking feature ---");
    let response3 = client
        .generate_content()
        .with_system_prompt("You are a helpful assistant.")
        .with_user_message("What is artificial intelligence?")
        .execute()
        .await?;

    println!("Answer: {}\n", response3.text());

    // Example 4: Use GenerationConfig to set thinking
    println!("--- Example 4: Use GenerationConfig to set thinking ---");
    let thinking_config = ThinkingConfig::new()
        .with_thinking_budget(2048)
        .with_thoughts_included(true);

    let generation_config = GenerationConfig {
        temperature: Some(0.7),
        max_output_tokens: Some(500),
        thinking_config: Some(thinking_config),
        ..Default::default()
    };

    let response4 = client
        .generate_content()
        .with_system_prompt("You are a creative writing assistant.")
        .with_user_message(
            "Write the opening of a short story about a robot learning to feel emotions.",
        )
        .with_generation_config(generation_config)
        .execute()
        .await?;

    // Display thinking process
    let thoughts4 = response4.thoughts();
    if !thoughts4.is_empty() {
        println!("Thinking summary:");
        for (i, thought) in thoughts4.iter().enumerate() {
            println!("Thought {}: {}\n", i + 1, thought);
        }
    }

    println!("Answer: {}\n", response4.text());

    Ok(())
}
