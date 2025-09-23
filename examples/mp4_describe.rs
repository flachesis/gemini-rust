// Please put your sample video at examples/sample.mp4
// This example sends the mp4 video content to Gemini API and asks AI to describe the video.

use base64::{Engine as _, engine::general_purpose};
use gemini_rust::{Content, Gemini};
use std::env;
use std::fs::File;
use std::io::Read;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read mp4 video file
    let mut file = File::open("examples/sample.mp4")?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let b64 = general_purpose::STANDARD.encode(&buffer);

    // Get API key
    let api_key = env::var("GEMINI_API_KEY")?;
    let gemini = Gemini::pro(api_key).expect("unable to create Gemini API client");

    // Example 1: Add mp4 blob using Message struct
    let video_content = Content::inline_data("video/mp4", b64.clone());
    let response1 = gemini
        .generate_content()
        .with_user_message("Please describe the content of this video (Message example)")
        .with_message(gemini_rust::Message {
            content: video_content,
            role: gemini_rust::Role::User,
        })
        .execute()
        .await?;

    println!("AI description (Message): {}", response1.text());

    // Example 2: Add mp4 blob directly using builder's with_inline_data
    let response2 = gemini
        .generate_content()
        .with_user_message("Please describe the content of this video (with_inline_data example)")
        .with_inline_data(b64, "video/mp4")
        .execute()
        .await?;

    println!("AI description (with_inline_data): {}", response2.text());
    Ok(())
}
