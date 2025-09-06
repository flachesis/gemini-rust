use futures::TryStreamExt;
use gemini_rust::{
    FunctionDeclaration, FunctionParameters, Gemini, PropertyDetails, ThinkingConfig,
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key from environment variable
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY environment variable not set");

    // Create client
    let client = Gemini::with_model(api_key, "models/gemini-2.5-pro".to_string())
        .expect("unable to create Gemini API client");

    println!("=== Gemini 2.5 Thinking Advanced Example ===\n");

    // Example 1: Streaming with thinking
    println!("--- Example 1: Streaming with thinking ---");
    let mut stream = client
        .generate_content()
        .with_system_prompt("You are a mathematics expert skilled at solving complex mathematical problems.")
        .with_user_message("Solve this math problem: Find the sum of the first 50 prime numbers. Please explain your solution process in detail.")
        .with_thinking_budget(2048)
        .with_thoughts_included(true)
        .execute_stream()
        .await?;

    println!("Streaming response:");
    let mut thoughts_shown = false;
    while let Some(chunk) = stream.try_next().await? {
        // Check if there's thinking content
        let thoughts = chunk.thoughts();
        if !thoughts.is_empty() && !thoughts_shown {
            println!("\nThinking process:");
            for (i, thought) in thoughts.iter().enumerate() {
                println!("Thought {}: {}", i + 1, thought);
            }
            println!("\nAnswer:");
            thoughts_shown = true;
        }

        // Display general text content
        print!("{}", chunk.text());
        std::io::Write::flush(&mut std::io::stdout())?;
    }
    println!("\n");

    // Example 2: Thinking combined with function calls
    println!("--- Example 2: Thinking combined with function calls ---");

    // Define a calculator function
    let calculator = FunctionDeclaration::new(
        "calculate",
        "Perform basic mathematical calculations",
        FunctionParameters::object()
            .with_property(
                "expression",
                PropertyDetails::string(
                    "The mathematical expression to calculate, e.g., '2 + 3 * 4'",
                ),
                true,
            )
            .with_property(
                "operation_type",
                PropertyDetails::enum_type("Type of calculation", ["arithmetic", "advanced"]),
                false,
            ),
    );

    let response = client
        .generate_content()
        .with_system_prompt("You are a mathematics assistant. When calculations are needed, use the provided calculator function.")
        .with_user_message("Calculate the result of (15 + 25) * 3 - 8 and explain the calculation steps.")
        .with_function(calculator)
        .with_thinking_budget(1024)
        .with_thoughts_included(true)
        .execute()
        .await?;

    // Display thinking process
    let thoughts = response.thoughts();
    if !thoughts.is_empty() {
        println!("Thinking process:");
        for (i, thought) in thoughts.iter().enumerate() {
            println!("Thought {}: {}\n", i + 1, thought);
        }
    }

    // Check for function calls
    let function_calls = response.function_calls();
    if !function_calls.is_empty() {
        println!("Function calls:");
        for (i, call) in function_calls.iter().enumerate() {
            println!("Call {}: {} Args: {}", i + 1, call.name, call.args);
        }
        println!();
    }

    println!("Answer: {}\n", response.text());

    // Example 3: Complex reasoning task
    println!("--- Example 3: Complex reasoning task ---");
    let complex_response = client
        .generate_content()
        .with_system_prompt("You are a logical reasoning expert.")
        .with_user_message(
            "There are three people: Alice, Bob, and Carol, who live in red, green, and blue houses respectively.\
            Given:\
            1. The person in the red house owns a cat\
            2. Bob does not live in the green house\
            3. Carol owns a dog\
            4. The green house is to the left of the red house\
            5. Alice does not own a cat\
            Please reason out which color house each person lives in and what pets they own.",
        )
        .with_thinking_config(
            ThinkingConfig::new()
                .with_thinking_budget(3072)
                .with_thoughts_included(true),
        )
        .execute()
        .await?;

    // Display thinking process
    let complex_thoughts = complex_response.thoughts();
    if !complex_thoughts.is_empty() {
        println!("Reasoning process:");
        for (i, thought) in complex_thoughts.iter().enumerate() {
            println!("Reasoning step {}: {}\n", i + 1, thought);
        }
    }

    println!("Conclusion: {}\n", complex_response.text());

    // Display token usage statistics
    if let Some(usage) = &complex_response.usage_metadata {
        println!("Token usage statistics:");
        println!("  Prompt tokens: {}", usage.prompt_token_count);
        println!("  Response tokens: {}", usage.candidates_token_count);
        if let Some(thinking_tokens) = usage.thoughts_token_count {
            println!("  Thinking tokens: {}", thinking_tokens);
        }
        println!("  Total tokens: {}", usage.total_token_count);
    }

    Ok(())
}
