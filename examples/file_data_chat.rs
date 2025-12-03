use display_error_chain::DisplayErrorChain;
use gemini_rust::{Content, Gemini};
use std::env;
use std::path::Path;
use std::process::ExitCode;
use tracing::info;

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt().init();

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
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY not set");
    let client = Gemini::new(api_key)?;

    let filename = "README.md";
    let file_path = Path::new(filename);

    info!(filename, "1. Reading local file...");
    let file_content = tokio::fs::read(file_path)
        .await
        .expect("Failed to read README.md");

    info!(size = file_content.len(), "2. Uploading file to Gemini...");

    let file_handle = client
        .create_file(file_content)
        .display_name("gemini_rust_readme.md")
        .with_mime_type("text/markdown".parse()?)
        .upload()
        .await?;

    info!(file_uri = ?file_handle.get_file_meta().uri, "File uploaded successfully");

    // Extract URI safely
    let file_uri = file_handle
        .get_file_meta()
        .uri
        .as_ref()
        .ok_or("File URI is missing")?
        .to_string();

    info!("3. Generating content using the uploaded file reference...");

    let response = client
        .generate_content()
        .with_file_data("text/markdown", file_uri)
        .with_user_message("Summarize the main features of this library based on the README file.")
        .execute()
        .await?;

    info!(response = response.text(), "Model response received");

    info!("4. Cleaning up remote file...");
    file_handle.delete().await.map_err(|e| e.1)?;

    Ok(())
}
