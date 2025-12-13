//! An example of uploading a file, and using it in a request.
use display_error_chain::DisplayErrorChain;
use gemini_rust::Gemini;
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
    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
    let gemini = Gemini::new(&api_key)?;

    info!("uploading a file with known content");

    // Upload a file
    let file_handle = gemini
        .create_file(fs::read("examples/sample.mp4")?)
        .display_name("sample.mp4")
        .with_mime_type("audio/mp4".parse()?)
        .upload()
        .await?;

    info!(file_name = file_handle.name(), "file uploaded successfully");

    // Use file in a request
    let response = gemini
        .generate_content()
        .with_system_prompt("You are a secretary.")
        .with_user_message_and_file("Summarize this meeting recording?", &file_handle)?
        .execute()
        .await?;

    info!(
        response = response.text(),
        "response with file content received"
    );

    info!("✅ Content generation example with file completed successfully!");
    Ok(())
}
