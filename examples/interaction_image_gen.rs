use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use display_error_chain::DisplayErrorChain;
use gemini_rust::prelude::*;
use std::fs;
use std::process::ExitCode;
use tracing::{info, warn};

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

    info!("image generation interaction example starting");

    let interaction = client
        .create_interaction()
        .with_model("gemini-3.1-flash-image")
        .with_text("Create a picture of a nano banana dish in a fancy restaurant with a Gemini theme. Photorealistic with elegant lighting.")
        .with_temperature(1.0)
        .execute()
        .await?;

    info!(response = interaction.output_text(), "text response");

    if let Some(InteractionContent::Image {
        data, mime_type, ..
    }) = interaction.output_image()
    {
        if let Some(ref image_data) = data {
            match BASE64.decode(image_data) {
                Ok(image_bytes) => {
                    let ext = match mime_type {
                        Some(ImageMimeType::Jpeg) => "jpg",
                        _ => "png",
                    };
                    let filename = format!("generated_interaction_image.{ext}");
                    fs::write(&filename, image_bytes)?;
                    info!(filename = filename, "image saved successfully");
                }
                Err(e) => {
                    warn!(error = ?e, "failed to decode image data");
                }
            }
        }
    }

    info!("image generation example completed");
    Ok(())
}
