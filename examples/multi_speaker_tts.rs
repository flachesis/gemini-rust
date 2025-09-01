use base64::{engine::general_purpose, Engine as _};
use gemini_rust::{Gemini, GenerationConfig, Part, SpeakerVoiceConfig, SpeechConfig};
use std::fs::File;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load API key from environment variable
    let api_key =
        std::env::var("GEMINI_API_KEY").expect("Please set GEMINI_API_KEY environment variable");

    // Create client with TTS-enabled model
    let client = Gemini::with_model(api_key, "models/gemini-2.5-flash-preview-tts".to_string())
        .expect("unable to create Gemini API client");

    println!("🎭 Gemini Multi-Speaker Speech Generation Example");
    println!("Generating multi-speaker audio from dialogue...\n");

    // Create multi-speaker configuration
    let speakers = vec![
        SpeakerVoiceConfig::new("Alice", "Puck"),
        SpeakerVoiceConfig::new("Bob", "Charon"),
    ];

    // Create generation config with multi-speaker speech settings
    let generation_config = GenerationConfig {
        response_modalities: Some(vec!["AUDIO".to_string()]),
        speech_config: Some(SpeechConfig::multi_speaker(speakers)),
        ..Default::default()
    };

    // Create a dialogue with speaker tags
    let dialogue = r#"
Alice: Hello there! I'm excited to demonstrate multi-speaker text-to-speech with Gemini.

Bob: That's amazing! I can't believe how natural this sounds. The different voices really bring the conversation to life.

Alice: Exactly! Each speaker has their own distinct voice characteristics, making it easy to follow who's speaking.

Bob: This technology opens up so many possibilities for audio content creation, educational materials, and accessibility features.

Alice: I couldn't agree more. It's remarkable how far AI-generated speech has come!
"#;

    match client
        .generate_content()
        .with_user_message(dialogue)
        .with_generation_config(generation_config)
        .execute()
        .await
    {
        Ok(response) => {
            println!("✅ Multi-speaker speech generation completed!");

            // Check if we have candidates
            for (i, candidate) in response.candidates.iter().enumerate() {
                if let Some(parts) = &candidate.content.parts {
                    for (j, part) in parts.iter().enumerate() {
                        match part {
                            // Look for inline data with audio MIME type
                            Part::InlineData { inline_data } => {
                                if inline_data.mime_type.starts_with("audio/") {
                                    println!("📄 Found audio data: {}", inline_data.mime_type);

                                    // Decode base64 audio data
                                    match general_purpose::STANDARD.decode(&inline_data.data) {
                                        Ok(audio_bytes) => {
                                            let filename =
                                                format!("multi_speaker_dialogue_{}_{}.pcm", i, j);

                                            // Save audio to file
                                            match File::create(&filename) {
                                                Ok(mut file) => {
                                                    if let Err(e) = file.write_all(&audio_bytes) {
                                                        eprintln!(
                                                            "❌ Error writing audio file: {}",
                                                            e
                                                        );
                                                    } else {
                                                        println!(
                                                            "💾 Multi-speaker audio saved as: {}",
                                                            filename
                                                        );
                                                        println!("🎧 Play with: aplay {} (Linux) or afplay {} (macOS)", filename, filename);
                                                        println!("👥 Features Alice (Puck voice) and Bob (Charon voice)");
                                                    }
                                                }
                                                Err(e) => {
                                                    eprintln!("❌ Error creating audio file: {}", e)
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("❌ Error decoding base64 audio: {}", e)
                                        }
                                    }
                                }
                            }
                            // Display any text content
                            Part::Text { text, thought } => {
                                if thought.unwrap_or(false) {
                                    println!("💭 Model thought: {}", text);
                                } else {
                                    println!("📝 Generated text: {}", text);
                                }
                            }
                            _ => {
                                // Handle other part types if needed
                            }
                        }
                    }
                }
            }

            // Display usage metadata if available
            if let Some(usage_metadata) = &response.usage_metadata {
                println!("\n📊 Usage Statistics:");
                println!("   Prompt tokens: {}", usage_metadata.prompt_token_count);
                println!("   Total tokens: {}", usage_metadata.total_token_count);
                if let Some(thoughts_tokens) = usage_metadata.thoughts_token_count {
                    println!("   Thinking tokens: {}", thoughts_tokens);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Error generating multi-speaker speech: {}", e);
            eprintln!("\n💡 Troubleshooting tips:");
            eprintln!("   1. Make sure GEMINI_API_KEY environment variable is set");
            eprintln!("   2. Verify you have access to the Gemini TTS model");
            eprintln!("   3. Check your internet connection");
            eprintln!("   4. Ensure speaker names in dialogue match configured speakers");
            eprintln!("   5. Make sure the model 'gemini-2.5-flash-preview-tts' supports multi-speaker TTS");
        }
    }

    println!("\n🎉 Example completed!");
    println!("💡 Tips for multi-speaker TTS:");
    println!("   • Use clear speaker names (Alice:, Bob:, etc.)");
    println!("   • Configure voice for each speaker beforehand");
    println!("   • Available voices: Puck, Charon, Kore, Fenrir, Aoede");
    println!("   • Each speaker maintains consistent voice characteristics");

    Ok(())
}
