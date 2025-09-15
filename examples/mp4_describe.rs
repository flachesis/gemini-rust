// Please put your sample video at examples/sample.mp4
// This example sends the mp4 video content to Gemini API and asks AI to describe the video.

use base64::{engine::general_purpose, Engine as _};
use gemini_rust::{Content, Gemini};
use std::env;
use std::fs::File;
use std::io::Read;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt::init();
    // Read mp4 video file
    info!("reading video file: examples/sample.mp4");
    let mut file = File::open("examples/sample.mp4")?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let b64 = general_purpose::STANDARD.encode(&buffer);
    info!(
        file_size = buffer.len(),
        "video file loaded and encoded to base64"
    );

    // Get API key
    let api_key = env::var("GEMINI_API_KEY")?;
    let gemini = Gemini::pro(api_key).expect("unable to create Gemini API client");

    // Example 1: Add mp4 blob using Message struct
    info!("starting video description example using Message struct");
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

    info!(
        description = response1.text(),
        "video description received using Message struct"
    );

    // Example 2: Add mp4 blob directly using builder's with_inline_data
    info!("starting video description example using with_inline_data");
    let response2 = gemini
        .generate_content()
        .with_user_message("Please describe the content of this video (with_inline_data example)")
        .with_inline_data(b64, "video/mp4")
        .execute()
        .await?;

    info!(
        description = response2.text(),
        "video description received using with_inline_data"
    );
    Ok(())
}
