/// Example demonstrating text responses with thoughtSignature support
///
/// This example shows how to handle text responses that include thought signatures,
/// as seen in the Gemini 2.5 Flash API response format.
use gemini_rust::{Content, GenerationResponse, Part};
use serde_json::json;
use tracing::info;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    info!("starting text with thoughtSignature example");

    // Simulate an API response similar to the one you provided
    let api_response = json!({
        "candidates": [
            {
                "content": {
                    "parts": [
                        {
                            "text": "**Okay, here's what I'm thinking:**\n\nThe user wants me to show them the functions available. I need to figure out what functions are accessible to me in this environment.",
                            "thought": true
                        },
                        {
                            "text": "The following functions are available in the environment: `chat.get_message_count()`",
                            "thoughtSignature": "Cs4BA.../Yw="
                        }
                    ],
                    "role": "model"
                },
                "finishReason": "STOP",
                "index": 0
            }
        ],
        "usageMetadata": {
            "promptTokenCount": 36,
            "candidatesTokenCount": 18,
            "totalTokenCount": 96,
            "promptTokensDetails": [
                {
                    "modality": "TEXT",
                    "tokenCount": 36
                }
            ],
            "thoughtsTokenCount": 42
        },
        "modelVersion": "gemini-2.5-flash",
        "responseId": "gIC..."
    });

    // Parse the response
    let response: GenerationResponse = serde_json::from_value(api_response)?;

    println!("📋 Parsed API Response:");
    println!(
        "Model Version: {}",
        response
            .model_version
            .as_ref()
            .unwrap_or(&"Unknown".to_string())
    );

    // Display usage metadata
    if let Some(usage) = &response.usage_metadata {
        println!("\n📊 Token Usage:");
        if let Some(prompt_token_count) = usage.prompt_token_count {
            println!("  Prompt tokens: {}", prompt_token_count);
        }
        println!(
            "  Response tokens: {}",
            usage.candidates_token_count.unwrap_or(0)
        );
        if let Some(total_token_count) = usage.total_token_count {
            println!("  Total tokens: {}", total_token_count);
        }
        if let Some(thinking_tokens) = usage.thoughts_token_count {
            println!("  Thinking tokens: {}", thinking_tokens);
        }
    }

    // Extract text parts with thought signatures using the new method
    println!("\n💭 Text Parts with Thought Analysis:");
    let text_with_thoughts = response.text_with_thoughts();

    for (i, (text, is_thought, thought_signature)) in text_with_thoughts.iter().enumerate() {
        println!("\n--- Part {} ---", i + 1);
        println!("Is thought: {}", is_thought);
        println!("Has thought signature: {}", thought_signature.is_some());

        if let Some(signature) = thought_signature {
            println!("Thought signature: {}", signature);
            println!("Signature length: {} characters", signature.len());
        }

        println!("Text content: {}", text);
    }

    // Demonstrate creating content with thought signatures
    println!("\n🔧 Creating Content with Thought Signatures:");

    let custom_content = Content::text_with_thought_signature(
        "This is a custom response with a thought signature",
        "custom_signature_abc123",
    );

    let custom_thought = Content::thought_with_signature(
        "This represents the model's thinking process",
        "thinking_signature_def456",
    );

    println!("Custom content JSON:");
    println!("{}", serde_json::to_string_pretty(&custom_content)?);

    println!("\nCustom thought JSON:");
    println!("{}", serde_json::to_string_pretty(&custom_thought)?);

    // Show how this would be used in multi-turn conversation context
    println!("\n🔄 Multi-turn Conversation Context:");
    println!("In a multi-turn conversation, you would include these parts");
    println!("with their thought signatures to maintain context:");

    // Extract the original parts for context preservation
    if let Some(candidate) = response.candidates.first() {
        if let Some(parts) = &candidate.content.parts {
            for (i, part) in parts.iter().enumerate() {
                if let Part::Text {
                    text: _,
                    thought,
                    thought_signature,
                } = part
                {
                    info!(
                        part_number = i + 1,
                        text_type = if *thought == Some(true) {
                            "Thought"
                        } else {
                            "Regular"
                        },
                        has_signature = thought_signature.is_some(),
                        "part analysis"
                    );

                    if let Some(sig) = thought_signature {
                        info!(
                            signature_preview = &sig[..10.min(sig.len())],
                            "preserve signature"
                        );
                    }
                }
            }
        }
    }

    info!("example completed successfully");
    Ok(())
}
