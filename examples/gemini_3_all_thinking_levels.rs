use gemini_rust::{prelude::*, ThinkingLevel};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");

    let gemini = Gemini::with_model(api_key, Model::Gemini3Flash)?;

    println!("=== Gemini 3 Flash: All Thinking Levels Demo ===\n");

    println!("Each thinking level demonstrates a different trade-off between");
    println!("response speed and reasoning depth.\n");

    let test_query = "What is the capital of France?";

    println!("=== Minimal Thinking Level ===");
    println!("Use for: Simple factual queries, fastest responses\n");

    let response_minimal = gemini
        .generate_content()
        .with_user_message(test_query)
        .with_thinking_level(ThinkingLevel::Minimal)
        .execute()
        .await?;

    println!("Query: {}", test_query);
    println!("Response: {}", response_minimal.text());

    if let Some(usage) = &response_minimal.usage_metadata {
        println!("Usage:");
        println!("  Prompt tokens: {:?}", usage.prompt_token_count);
        println!("  Response tokens: {:?}", usage.candidates_token_count);
        println!("  Thought tokens: {:?}", usage.thoughts_token_count);
        println!("  Total tokens: {:?}", usage.total_token_count);
    }

    println!("\n=== Low Thinking Level ===");
    println!("Use for: Straightforward questions requiring some reasoning\n");

    let query_low = "What is 15% of 200?";

    let response_low = gemini
        .generate_content()
        .with_user_message(query_low)
        .with_thinking_level(ThinkingLevel::Low)
        .execute()
        .await?;

    println!("Query: {}", query_low);
    println!("Response: {}", response_low.text());

    if let Some(usage) = &response_low.usage_metadata {
        println!("Usage:");
        println!("  Prompt tokens: {:?}", usage.prompt_token_count);
        println!("  Response tokens: {:?}", usage.candidates_token_count);
        println!("  Thought tokens: {:?}", usage.thoughts_token_count);
        println!("  Total tokens: {:?}", usage.total_token_count);
    }

    println!("\n=== Medium Thinking Level ===");
    println!("Use for: Moderately complex tasks requiring balanced analysis\n");

    let query_medium = "Compare and contrast the advantages and disadvantages of electric cars versus gas-powered cars.";

    let response_medium = gemini
        .generate_content()
        .with_user_message(query_medium)
        .with_thinking_level(ThinkingLevel::Medium)
        .with_thoughts_included(true)
        .execute()
        .await?;

    println!("Query: {}", query_medium);
    println!("Response: {}", response_medium.text());

    if !response_medium.thoughts().is_empty() {
        println!("\nThoughts:");
        for thought in response_medium.thoughts() {
            println!("  - {}", thought);
        }
    }

    if let Some(usage) = &response_medium.usage_metadata {
        println!("Usage:");
        println!("  Prompt tokens: {:?}", usage.prompt_token_count);
        println!("  Response tokens: {:?}", usage.candidates_token_count);
        println!("  Thought tokens: {:?}", usage.thoughts_token_count);
        println!("  Total tokens: {:?}", usage.total_token_count);
    }

    println!("\n=== High Thinking Level ===");
    println!("Use for: Complex analytical tasks requiring deep reasoning\n");

    let query_high = "Explain the implications of quantum computing on modern cryptography, including both the potential vulnerabilities and the proposed solutions.";

    let response_high = gemini
        .generate_content()
        .with_user_message(query_high)
        .with_thinking_level(ThinkingLevel::High)
        .with_thoughts_included(true)
        .execute()
        .await?;

    println!("Query: {}", query_high);
    println!("Response: {}", response_high.text());

    if !response_high.thoughts().is_empty() {
        println!("\nThoughts:");
        for thought in response_high.thoughts() {
            println!("  - {}", thought);
        }
    }

    if let Some(usage) = &response_high.usage_metadata {
        println!("Usage:");
        println!("  Prompt tokens: {:?}", usage.prompt_token_count);
        println!("  Response tokens: {:?}", usage.candidates_token_count);
        println!("  Thought tokens: {:?}", usage.thoughts_token_count);
        println!("  Total tokens: {:?}", usage.total_token_count);
    }

    println!("\n=== Thinking Level Comparison ===");
    println!("| Level      | Reasoning Depth | Speed     | Best Use Case                    |");
    println!("|------------|-----------------|-----------|----------------------------------|");
    println!("| Minimal    | Minimal         | Fastest   | Simple facts, quick answers      |");
    println!("| Low        | Low             | Fast      | Straightforward calculations     |");
    println!("| Medium     | Balanced        | Moderate  | Comparative analysis, summaries  |");
    println!("| High       | Deep            | Slow      | Complex reasoning, explanations |");
    println!("| Unspecified| Model default   | Variable  | Let model decide                |");

    println!("\n=== Tips for Choosing Thinking Levels ===");
    println!("- Start with Minimal or Low for simple queries to minimize cost");
    println!("- Use Medium for multi-step reasoning or comparison tasks");
    println!("- Use High for complex problems requiring detailed analysis");
    println!("- Enable with_thoughts_included(true) with Medium/High for insights");
    println!("- Adjust based on your latency vs. accuracy requirements");

    Ok(())
}
