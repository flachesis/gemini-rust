use gemini_rust::{prelude::*, Part, ThinkingLevel};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");

    let gemini = Gemini::with_model(api_key, Model::Gemini3Pro)?;

    println!("=== Gemini 3 Pro: Code Execution Demo ===\n");

    let response = gemini
        .generate_content()
        .with_user_message("Calculate the 50th Fibonacci number using Python code. Show your work.")
        .with_code_execution()
        .with_thinking_level(ThinkingLevel::High)
        .with_thoughts_included(true)
        .execute()
        .await?;

    println!("Response:");
    println!("{}", response.text());

    if !response.thoughts().is_empty() {
        println!("\nThoughts:");
        for thought in response.thoughts() {
            println!("  - {thought}");
        }
    }

    if let Some(first_candidate) = response.candidates.first() {
        if let Some(parts) = &first_candidate.content.parts {
            println!("\n=== Checking for Code Execution Parts ===");
            for (i, part) in parts.iter().enumerate() {
                match part {
                    Part::ExecutableCode { executable_code } => {
                        println!("\nExecutable Code Part #{}:", i + 1);
                        println!("  Language: {:?}", executable_code.language);
                        println!("  Code:\n{}", executable_code.code);
                    }
                    Part::CodeExecutionResult { code_execution_result } => {
                        println!("\nCode Execution Result Part #{}:", i + 1);
                        println!("  Outcome: {:?}", code_execution_result.outcome);
                        println!("  Output:\n{}", code_execution_result.output);
                    }
                    Part::Text { text, thought, .. } => {
                        if thought.unwrap_or(false) {
                            println!("\nThought Part #{}:", i + 1);
                            println!("  {text}");
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    if let Some(usage) = &response.usage_metadata {
        println!("\n=== Usage Metadata ===");
        println!("  Prompt tokens: {:?}", usage.prompt_token_count);
        println!("  Response tokens: {:?}", usage.candidates_token_count);
        println!("  Thought tokens: {:?}", usage.thoughts_token_count);
        println!("  Total tokens: {:?}", usage.total_token_count);
    }

    println!("\n=== Gemini 3 Pro: Prime Number Calculation ===\n");

    let response_prime = gemini
        .generate_content()
        .with_user_message("Calculate the first 20 prime numbers using Python code.")
        .with_code_execution()
        .with_thinking_level(ThinkingLevel::Low)
        .execute()
        .await?;

    println!("Response:");
    println!("{}", response_prime.text());

    if let Some(first_candidate) = response_prime.candidates.first() {
        if let Some(parts) = &first_candidate.content.parts {
            for part in parts {
                if let Part::CodeExecutionResult { code_execution_result } = part {
                    println!("\nCode Execution Output:");
                    println!("{}", code_execution_result.output);
                    println!("Outcome: {:?}", code_execution_result.outcome);
                }
            }
        }
    }

    println!("\n=== Gemini 3 Pro: Data Analysis with Code ===\n");

    let response_analysis = gemini
        .generate_content()
        .with_user_message("Generate a list of 100 random numbers and calculate the mean, median, and standard deviation using Python.")
        .with_code_execution()
        .with_thinking_level(ThinkingLevel::High)
        .with_thoughts_included(true)
        .execute()
        .await?;

    println!("Response:");
    println!("{}", response_analysis.text());

    if !response_analysis.thoughts().is_empty() {
        println!("\nThoughts:");
        for thought in response_analysis.thoughts() {
            println!("  - {thought}");
        }
    }

    if let Some(first_candidate) = response_analysis.candidates.first() {
        if let Some(parts) = &first_candidate.content.parts {
            for part in parts {
                match part {
                    Part::ExecutableCode { executable_code } => {
                        println!("\nGenerated Code:");
                        println!("{}", executable_code.code);
                    }
                    Part::CodeExecutionResult { code_execution_result } => {
                        println!("\nExecution Result:");
                        println!("{}", code_execution_result.output);
                    }
                    _ => {}
                }
            }
        }
    }

    println!("\n=== Gemini 3 Pro: Complex Calculation ===\n");

    let response_complex = gemini
        .generate_content()
        .with_user_message(
            "Calculate the definite integral of x^2 from 0 to 10 using numerical integration in Python. \
             Compare it with the analytical result."
        )
        .with_code_execution()
        .with_thinking_level(ThinkingLevel::High)
        .with_thoughts_included(true)
        .execute()
        .await?;

    println!("Response:");
    println!("{}", response_complex.text());

    if !response_complex.thoughts().is_empty() {
        println!("\nThoughts:");
        for thought in response_complex.thoughts() {
            println!("  - {thought}");
        }
    }

    if let Some(usage) = &response_complex.usage_metadata {
        println!("\n=== Usage Metadata ===");
        println!("  Prompt tokens: {:?}", usage.prompt_token_count);
        println!("  Response tokens: {:?}", usage.candidates_token_count);
        println!("  Thought tokens: {:?}", usage.thoughts_token_count);
        println!("  Total tokens: {:?}", usage.total_token_count);
    }

    Ok(())
}
