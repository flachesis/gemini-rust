/// Comprehensive example demonstrating thoughtSignature support in Gemini 2.5 Pro
///
/// This example shows:
/// 1. How to enable thinking and function calling to receive thought signatures
/// 2. How to extract thought signatures from function call responses
/// 3. How to maintain thought context across multiple turns in a conversation
///
/// Key points about thought signatures:
/// - Only available with Gemini 2.5 series models
/// - Requires both thinking and function calling to be enabled
/// - Must include the entire response with thought signatures in subsequent turns
/// - Don't concatenate or merge parts with signatures
///
/// Thought signatures are encrypted representations of the model's internal
/// thought process that help maintain context across conversation turns.
use gemini_rust::{
    FunctionCallingMode, FunctionDeclaration, FunctionParameters, FunctionResponse, Gemini,
    PropertyDetails, ThinkingConfig, Tool,
};
use serde_json::json;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key from environment variable
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY environment variable not set");

    // Create client using Gemini 2.5 Pro which supports thoughtSignature
    let client = Gemini::pro(api_key).expect("unable to create Gemini API client");

    println!("=== Gemini 2.5 Pro thoughtSignature Example ===\n");

    // Define the weather function tool
    let weather_function = FunctionDeclaration::new(
        "get_current_weather",
        "Get current weather information for a specified location",
        FunctionParameters::object().with_property(
            "location",
            PropertyDetails::string("City and region, e.g., Kaohsiung Zuoying District"),
            true,
        ),
    );

    let weather_tool = Tool::new(weather_function);

    // Configure thinking to enable thoughtSignature
    let thinking_config = ThinkingConfig::new()
        .with_dynamic_thinking()
        .with_thoughts_included(true);

    // First request: Ask about weather (expecting function call with thoughtSignature)
    println!("--- Step 1: Asking about weather (expecting function call) ---");

    let response = client
        .generate_content()
        .with_system_instruction("Please respond in Traditional Chinese")
        .with_user_message("What's the weather like in Kaohsiung Zuoying District right now?")
        .with_tool(weather_tool)
        .with_function_calling_mode(FunctionCallingMode::Auto)
        .with_thinking_config(thinking_config)
        .execute()
        .await?;

    // Check for function calls with thought signatures
    let function_calls_with_thoughts = response.function_calls_with_thoughts();

    if !function_calls_with_thoughts.is_empty() {
        println!("Function calls received:");
        for (function_call, thought_signature) in function_calls_with_thoughts {
            println!("  Function: {}", function_call.name);
            println!("  Arguments: {}", function_call.args);

            if let Some(signature) = thought_signature {
                println!("  Thought Signature: {}", signature);
                println!("  Thought Signature Length: {} characters", signature.len());
            } else {
                println!("  No thought signature provided");
            }
            println!();

            // Mock function response
            let weather_data = json!({
                "temperature": "25°C",
                "condition": "sunny",
                "humidity": "60%",
                "wind": "light breeze",
                "location": function_call.get::<String>("location").unwrap_or_default()
            });

            println!("Mock weather response: {}", weather_data);
            println!();

            // Continue the conversation with function response
            println!("--- Step 2: Providing function response ---");
            let function_response = FunctionResponse::new(&function_call.name, weather_data);

            let final_response = client
                .generate_content()
                .with_system_instruction("Please respond in Traditional Chinese")
                .with_user_message(
                    "What's the weather like in Kaohsiung Zuoying District right now?",
                )
                .with_function_response(
                    &function_call.name,
                    function_response.response.unwrap_or_default(),
                )
                .execute()
                .await?;

            println!("Final response: {}", final_response.text());

            // Display usage metadata
            if let Some(usage) = &final_response.usage_metadata {
                println!("\nToken usage:");
                if let Some(prompt_tokens) = usage.prompt_token_count {
                    println!("  Prompt tokens: {}", prompt_tokens);
                }
                if let Some(response_tokens) = usage.candidates_token_count {
                    println!("  Response tokens: {}", response_tokens);
                }
                if let Some(thinking_tokens) = usage.thoughts_token_count {
                    println!("  Thinking tokens: {}", thinking_tokens);
                }
                if let Some(total_tokens) = usage.total_token_count {
                    println!("  Total tokens: {}", total_tokens);
                }
            }

            // --- Step 3: Multi-turn conversation with thought context ---
            println!("\n--- Step 3: Multi-turn conversation maintaining thought context ---");
            println!("IMPORTANT: To maintain thought context, we must include the complete");
            println!("previous response with thought signatures in the next turn.");

            // Create a multi-turn conversation that includes the previous context
            // We need to include ALL parts from the previous responses to maintain thought context
            let mut conversation_builder = client.generate_content();

            // Add system instruction
            conversation_builder = conversation_builder
                .with_system_instruction("Please respond in Traditional Chinese");

            // Add the original user message
            conversation_builder = conversation_builder.with_user_message(
                "What's the weather like in Kaohsiung Zuoying District right now?",
            );

            // IMPORTANT: Add the model's response with the function call INCLUDING the thought signature
            // This maintains the thought context for the next turn
            // DO NOT concatenate parts or merge signatures - include the complete original part
            let model_content = gemini_rust::Content {
                parts: Some(vec![gemini_rust::Part::FunctionCall {
                    function_call: function_call.clone(),
                    thought_signature: thought_signature.cloned(), // This is crucial for context
                }]),
                role: Some(gemini_rust::Role::Model),
            };
            conversation_builder.contents.push(model_content);

            // Add the function response
            conversation_builder = conversation_builder.with_function_response(
                &function_call.name,
                json!({
                    "temperature": "25°C",
                    "condition": "sunny",
                    "humidity": "60%",
                    "wind": "light breeze",
                    "location": function_call.get::<String>("location").unwrap_or_default()
                }),
            );

            // Add the model's text response (complete the conversation history)
            let model_text_content = gemini_rust::Content {
                parts: Some(vec![gemini_rust::Part::Text {
                    text: final_response.text(),
                    thought: None,
                }]),
                role: Some(gemini_rust::Role::Model),
            };
            conversation_builder.contents.push(model_text_content);

            // Now ask a follow-up question that can benefit from the thought context
            // The model will have access to its previous reasoning through the thought signature
            conversation_builder = conversation_builder.with_user_message("Is this weather suitable for outdoor sports? Please recommend some appropriate activities.");

            // Add the weather tool again for potential follow-up function calls
            let weather_tool_followup = Tool::new(FunctionDeclaration::new(
                "get_current_weather",
                "Get current weather information for a specified location",
                FunctionParameters::object().with_property(
                    "location",
                    PropertyDetails::string("City and region, e.g., Kaohsiung Zuoying District"),
                    true,
                ),
            ));

            conversation_builder = conversation_builder
                .with_tool(weather_tool_followup)
                .with_function_calling_mode(FunctionCallingMode::Auto)
                .with_thinking_config(
                    ThinkingConfig::new()
                        .with_dynamic_thinking()
                        .with_thoughts_included(true),
                );

            let followup_response = conversation_builder.execute().await?;

            println!("Follow-up question: Is this weather suitable for outdoor sports? Please recommend some appropriate activities.");
            println!("Follow-up response: {}", followup_response.text());

            // Check if there are any new function calls with thought signatures in the follow-up
            let followup_function_calls = followup_response.function_calls_with_thoughts();
            if !followup_function_calls.is_empty() {
                println!("\nFollow-up function calls:");
                for (fc, ts) in followup_function_calls {
                    println!("  Function: {}", fc.name);
                    println!("  Arguments: {}", fc.args);
                    if let Some(sig) = ts {
                        println!("  New thought signature: {} characters", sig.len());
                    }
                }
            }

            // Display thinking process for follow-up
            let followup_thoughts = followup_response.thoughts();
            if !followup_thoughts.is_empty() {
                println!("\nFollow-up thinking summaries:");
                for (i, thought) in followup_thoughts.iter().enumerate() {
                    println!("  Follow-up thought {}: {}", i + 1, thought);
                }
            }

            // Display follow-up usage metadata
            if let Some(usage) = &followup_response.usage_metadata {
                println!("\nFollow-up token usage:");
                if let Some(prompt_tokens) = usage.prompt_token_count {
                    println!("  Prompt tokens: {}", prompt_tokens);
                }
                if let Some(response_tokens) = usage.candidates_token_count {
                    println!("  Response tokens: {}", response_tokens);
                }
                if let Some(thinking_tokens) = usage.thoughts_token_count {
                    println!("  Thinking tokens: {}", thinking_tokens);
                }
                if let Some(total_tokens) = usage.total_token_count {
                    println!("  Total tokens: {}", total_tokens);
                }
            }

            println!("\n=== Multi-turn conversation completed ===");
            println!("Key takeaways:");
            println!("1. Thought signatures help maintain context across conversation turns");
            println!("2. Include the complete response (with signatures) in subsequent requests");
            println!("3. Don't modify or concatenate parts that contain thought signatures");
            println!("4. Thought signatures are only available with thinking + function calling");
            println!(
                "5. The model can build upon its previous reasoning when signatures are preserved"
            );
        }
    } else {
        println!("No function calls in response");
        println!("Response text: {}", response.text());
    }

    // Display any thoughts from the initial response
    let thoughts = response.thoughts();
    if !thoughts.is_empty() {
        println!("\nInitial thinking summaries:");
        for (i, thought) in thoughts.iter().enumerate() {
            println!("  Thought {}: {}", i + 1, thought);
        }
    }

    Ok(())
}
