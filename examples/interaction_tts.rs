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
    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY environment variable not set");
    let client = Gemini::new(api_key)?;

    info!("text-to-speech interaction example starting");

    let interaction = client
        .create_interaction()
        .with_model("gemini-3.1-flash-tts-preview")
        .with_text("Say hello in a cheerful voice: 'Hello! Welcome to the world of AI!'")
        .with_speech_config(InteractionSpeechConfig {
            voice: Some("Charon".to_string()),
            language: None,
            speaker: None,
        })
        .execute()
        .await?;

    info!(response = interaction.output_text(), "text response");

    if let Some(InteractionContent::Audio { data, mime_type, .. }) = interaction.output_audio() {
        if let Some(ref audio_data) = data {
            let audio_bytes = BASE64.decode(audio_data)?;
            let ext = match mime_type {
                Some(AudioMimeType::Mp3) => "mp3",
                Some(AudioMimeType::Wav) => "wav",
                _ => "pcm",
            };
            let filename = format!("generated_speech.{ext}");
            fs::write(&filename, audio_bytes)?;
            info!(filename = filename, "audio saved");
        }
    }

    info!("text-to-speech example completed");
    Ok(())
}
