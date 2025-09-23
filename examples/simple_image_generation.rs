use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use gemini_rust::{Gemini, GenerationConfig};
use std::env;
use std::fs;

/// Simple image generation example
/// This demonstrates the basic usage of Gemini's image generation capabilities
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key from environment variable
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY environment variable not set");

    // Create client with the image generation model
    // Use Gemini 2.5 Flash Image Preview for image generation
    let client = Gemini::with_model(api_key, "models/gemini-2.5-flash-image-preview".to_string())
        .expect("unable to create Gemini API client");

    println!("🎨 Generating image with Gemini...");

    // Generate an image from text description
    let response = client
        .generate_content()
        .with_user_message(
            "Create a photorealistic image of a cute robot sitting in a garden, \
             surrounded by colorful flowers. The robot should have a friendly \
             expression and be made of polished metal. The lighting should be \
             soft and natural, as if taken during golden hour.",
        )
        .with_generation_config(GenerationConfig {
            temperature: Some(0.8),
            max_output_tokens: Some(8192),
            ..Default::default()
        })
        .execute()
        .await?;

    // Process the response
    let mut images_saved = 0;
    for candidate in response.candidates.iter() {
        if let Some(parts) = &candidate.content.parts {
            for part in parts.iter() {
                match part {
                    gemini_rust::Part::Text { text, .. } => {
                        println!("📝 Model response: {}", text);
                    }
                    gemini_rust::Part::InlineData { inline_data } => {
                        println!("🖼️  Image generated!");
                        println!("   MIME type: {}", inline_data.mime_type);

                        // Decode and save the image
                        match BASE64.decode(&inline_data.data) {
                            Ok(image_bytes) => {
                                images_saved += 1;
                                let filename = format!("robot_garden_{}.png", images_saved);
                                fs::write(&filename, image_bytes)?;
                                println!("✅ Image saved as: {}", filename);
                            }
                            Err(e) => {
                                println!("❌ Failed to decode image: {}", e);
                            }
                        }
                    }
                    _ => {
                        println!("🔍 Other content type in response");
                    }
                }
            }
        }
    }

    if images_saved == 0 {
        println!("⚠️  No images were generated. This might be due to:");
        println!("   - Content policy restrictions");
        println!("   - API limitations");
        println!("   - Model configuration issues");
    } else {
        println!("🎉 Successfully generated {} image(s)!", images_saved);
    }

    Ok(())
}
