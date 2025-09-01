use gemini_rust::{Gemini, GenerationConfig, ThinkingConfig};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key from environment variable
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY environment variable not set");

    // This is equivalent to the following curl example:
    // curl "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-pro:generateContent" \
    //   -H "x-goog-api-key: $GEMINI_API_KEY" \
    //   -H 'Content-Type: application/json' \
    //   -X POST \
    //   -d '{
    //     "contents": [
    //       {
    //         "parts": [
    //           {
    //             "text": "Provide a list of the top 3 famous physicists and their major contributions"
    //           }
    //         ]
    //       }
    //     ],
    //     "generationConfig": {
    //       "thinkingConfig": {
    //         "thinkingBudget": 1024,
    //         "includeThoughts": true
    //       }
    //     }
    //   }'

    // Create client
    let client = Gemini::with_model(api_key, "models/gemini-2.5-pro".to_string())
        .expect("unable to create Gemini API client");

    println!("=== Thinking Curl Equivalent Example ===\n");

    // Method 1: Using high-level API (simplest approach)
    println!("--- Method 1: Using high-level API ---");

    let response1 = client
        .generate_content()
        .with_user_message(
            "Provide a list of the top 3 famous physicists and their major contributions",
        )
        .with_thinking_budget(1024)
        .with_thoughts_included(true)
        .execute()
        .await?;

    // Display thinking process
    let thoughts1 = response1.thoughts();
    if !thoughts1.is_empty() {
        println!("Thinking summary:");
        for (i, thought) in thoughts1.iter().enumerate() {
            println!("Thought {}: {}\n", i + 1, thought);
        }
    }

    println!("Answer: {}\n", response1.text());

    // Method 2: Using GenerationConfig to fully match curl example structure
    println!("--- Method 2: Fully matching curl example structure ---");

    let thinking_config = ThinkingConfig {
        thinking_budget: Some(1024),
        include_thoughts: Some(true),
    };

    let generation_config = GenerationConfig {
        thinking_config: Some(thinking_config),
        ..Default::default()
    };

    let response2 = client
        .generate_content()
        .with_user_message(
            "Provide a list of the top 3 famous physicists and their major contributions",
        )
        .with_generation_config(generation_config)
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

    // Show token usage
    if let Some(usage) = &response2.usage_metadata {
        println!("Token usage:");
        println!("  Prompt tokens: {}", usage.prompt_token_count);
        println!(
            "  Response tokens: {}",
            usage.candidates_token_count.unwrap_or(0)
        );
        if let Some(thinking_tokens) = usage.thoughts_token_count {
            println!("  Thinking tokens: {}", thinking_tokens);
        }
        println!("  Total tokens: {}", usage.total_token_count);
    }

    // Method 3: Demonstrate different thinking budget settings
    println!("\n--- Method 3: Different thinking budget comparison ---");

    // Thinking disabled
    println!("Thinking disabled:");
    let response_no_thinking = client
        .generate_content()
        .with_user_message("Explain the basic principles of quantum mechanics")
        .execute()
        .await?;
    println!("Answer: {}\n", response_no_thinking.text());

    // Dynamic thinking
    println!("Dynamic thinking:");
    let response_dynamic = client
        .generate_content()
        .with_user_message("Explain the basic principles of quantum mechanics")
        .with_dynamic_thinking()
        .with_thoughts_included(true)
        .execute()
        .await?;

    let thoughts_dynamic = response_dynamic.thoughts();
    if !thoughts_dynamic.is_empty() {
        println!("Thinking summary:");
        for (i, thought) in thoughts_dynamic.iter().enumerate() {
            println!("Thought {}: {}\n", i + 1, thought);
        }
    }
    println!("Answer: {}\n", response_dynamic.text());

    // High thinking budget
    println!("High thinking budget (4096 tokens):");
    let response_high_budget = client
        .generate_content()
        .with_user_message("Explain the basic principles of quantum mechanics")
        .with_thinking_budget(4096)
        .with_thoughts_included(true)
        .execute()
        .await?;

    let thoughts_high = response_high_budget.thoughts();
    if !thoughts_high.is_empty() {
        println!("Thinking summary:");
        for (i, thought) in thoughts_high.iter().enumerate() {
            println!("Thought {}: {}\n", i + 1, thought);
        }
    }
    println!("Answer: {}", response_high_budget.text());

    Ok(())
}
