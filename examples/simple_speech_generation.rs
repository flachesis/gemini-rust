use base64::{engine::general_purpose, Engine as _};
use gemini_rust::{Gemini, GenerationConfig, Part, PrebuiltVoiceConfig, SpeechConfig, VoiceConfig};
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

    println!("🎤 Gemini Speech Generation Example");
    println!("Generating audio from text...\n");

    // Create generation config with speech settings
    let generation_config = GenerationConfig {
        response_modalities: Some(vec!["AUDIO".to_string()]),
        speech_config: Some(SpeechConfig {
            voice_config: Some(VoiceConfig {
                prebuilt_voice_config: Some(PrebuiltVoiceConfig {
                    voice_name: "Puck".to_string(),
                }),
            }),
            multi_speaker_voice_config: None,
        }),
        ..Default::default()
    };

    match client
        .generate_content()
        .with_user_message("Hello! This is a demonstration of text-to-speech using Google's Gemini API. The voice you're hearing is generated entirely by AI.")
        .with_generation_config(generation_config)
        .execute()
        .await {
        Ok(response) => {
            println!("✅ Speech generation completed!");

            // Check if we have candidates
            for (i, candidate) in response.candidates.iter().enumerate() {
                if let Some(parts) = &candidate.content.parts {
                    for (j, part) in parts.iter().enumerate() {
                        match part {
                            // Look for inline data with audio MIME type
                            Part::InlineData { inline_data } => {
                                if inline_data.mime_type.starts_with("audio/") {
                                    println!("📄 Found audio data: {}", inline_data.mime_type);

                                    // Decode base64 audio data using the new API
                                    match general_purpose::STANDARD.decode(&inline_data.data) {
                                        Ok(audio_bytes) => {
                                            let filename = format!("speech_output_{}_{}.pcm", i, j);

                                            // Save audio to file
                                            match File::create(&filename) {
                                                Ok(mut file) => {
                                                    if let Err(e) = file.write_all(&audio_bytes) {
                                                        eprintln!("❌ Error writing audio file: {}", e);
                                                    } else {
                                                        println!("💾 Audio saved as: {}", filename);
                                                        println!("🔊 You can play it with: aplay {} (Linux) or afplay {} (macOS)", filename, filename);
                                                    }
                                                },
                                                Err(e) => eprintln!("❌ Error creating audio file: {}", e),
                                            }
                                        },
                                        Err(e) => eprintln!("❌ Error decoding base64 audio: {}", e),
                                    }
                                }
                            },
                            // Display any text content
                            Part::Text { text, thought } => {
                                if thought.unwrap_or(false) {
                                    println!("💭 Thought: {}", text);
                                } else {
                                    println!("📝 Text content: {}", text);
                                }
                            },
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
                if let Some(prompt_tokens) = usage_metadata.prompt_token_count {
                    println!("   Prompt tokens: {}", prompt_tokens);
                }
                if let Some(total_tokens) = usage_metadata.total_token_count {
                    println!("   Total tokens: {}", total_tokens);
                }
            }
        },
        Err(e) => {
            eprintln!("❌ Error generating speech: {}", e);
            eprintln!("\n💡 Troubleshooting tips:");
            eprintln!("   1. Make sure GEMINI_API_KEY environment variable is set");
            eprintln!("   2. Verify you have access to the Gemini TTS model");
            eprintln!("   3. Check your internet connection");
            eprintln!("   4. Ensure the model 'gemini-2.5-flash-preview-tts' is available");
        }
    }

    Ok(())
}
