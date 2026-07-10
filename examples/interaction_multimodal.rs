use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use display_error_chain::DisplayErrorChain;
use gemini_rust::prelude::*;
use std::fs;
use std::process::ExitCode;
use tracing::info;

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing::level_filters::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    match do_main().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            let error_chain = DisplayErrorChain::new(e.as_ref());
            tracing::error!(error.debug = ?e, error.chained = %error_chain, "execution failed");
            ExitCode::FAILURE
        }
    }
}

async fn do_main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key =
        std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY environment variable not set");
    let client = Gemini::new(api_key)?;

    info!("multimodal interaction example starting");

    let image_data = fs::read("examples/image-example.webp")?;
    let image_b64 = BASE64.encode(&image_data);

    let interaction = client
        .create_interaction()
        .with_model("gemini-flash-latest")
        .with_text("What's in this image?")
        .with_image(image_b64, ImageMimeType::Webp)
        .execute()
        .await?;

    info!(response = interaction.output_text(), "image analysis");

    info!("multimodal example with video URI");

    let video_interaction = client
        .create_interaction()
        .with_model("gemini-flash-latest")
        .with_text("Describe this video in detail.")
        .with_video("https://storage.googleapis.com/generativeai-downloads/images/GreatRedSpot.mp4")
        .execute()
        .await?;

    info!(response = video_interaction.output_text(), "video analysis");

    info!("multimodal example completed");
    Ok(())
}
