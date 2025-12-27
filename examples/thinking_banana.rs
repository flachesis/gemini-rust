use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use display_error_chain::DisplayErrorChain;
use gemini_rust::{Gemini, GenerationConfig, Part};
use std::process::ExitCode;
use std::{env, fs};
use tracing::level_filters::LevelFilter;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
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
    // Get API key from environment variable
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY environment variable not set");

    // Create client
    let client = Gemini::pro_image(api_key).expect("unable to create Gemini API client");

    info!("starting gemini 3.0 pro image thinking basic example");

    // Example 1: Using default dynamic thinking
    info!("example 1: dynamic thinking (model automatically determines thinking budget)");
    let response1 = client
        .generate_content()
        .with_system_prompt("You are a world class watercolor painting assistant.")
        .with_user_message(
            r#"Draw a scene in a watercolor style that includes the steps required to create your 
            first commit on GitHub. Each step is a scene with using little water color people that is
            the scenario with text explaining the command prompts needed. 
            Include a person sitting at a desk with a CRT monitor at a 
            command line interface, typing git commands. Show the GitHub logo prominently 
            in the background."#,
        )
        .with_generation_config(GenerationConfig {
            temperature: Some(1.0),
            max_output_tokens: Some(32768),
            ..Default::default()
        })
        .with_dynamic_thinking()
        .with_thoughts_included(true)
        .execute()
        .await?;

    // Display thinking process
    let thoughts = response1.thoughts();
    if !thoughts.is_empty() {
        info!("showing thinking summary");
        for (i, thought) in thoughts.iter().enumerate() {
            info!(thought_number = i + 1, thought = thought, "thought");
        }
    }

    // Process the response
    let mut images_saved = 0;
    for candidate in response1.candidates.iter() {
        if let Some(parts) = &candidate.content.parts {
            for part in parts.iter() {
                match part {
                    Part::Text { text, .. } => {
                        info!(response = text, "model text response received");
                    }
                    Part::InlineData { inline_data, .. } => {
                        info!(mime_type = inline_data.mime_type, "image generated");

                        // Decode and save the image
                        match BASE64.decode(&inline_data.data) {
                            Ok(image_bytes) => {
                                images_saved += 1;
                                let filename = format!("git_poster_{images_saved}.png");
                                fs::write(&filename, image_bytes)?;
                                info!(filename = filename, "image saved successfully");
                            }
                            Err(e) => {
                                warn!(error = ?e, "failed to decode image");
                            }
                        }
                    }
                    _ => {
                        info!("other content type found in response");
                    }
                }
            }
        }
    }

    if images_saved == 0 {
        warn!("no images were generated - possible reasons: content policy restrictions, API limitations, or model configuration issues");
    } else {
        info!(
            images_count = images_saved,
            "image generation completed successfully"
        );
    }

    Ok(())
}
