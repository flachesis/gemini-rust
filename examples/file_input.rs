//! Demonstrates using file handles in content generation requests.
//!
//! This example shows how to upload a file and reference it in a generation request
//! using the file handle, which avoids re-encoding and re-sending large files.

use display_error_chain::DisplayErrorChain;
use gemini_rust::Gemini;
use std::env;
use std::process::ExitCode;
use tracing::{error, info};

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
            error!(error.debug = ?e, error.chained = %error_chain, "execution failed");
            ExitCode::FAILURE
        }
    }
}

async fn do_main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY not set");
    let client = Gemini::new(api_key)?;

    // Create a minimal valid PDF for testing
    // In a real application, you would read an actual file from disk
    let dummy_pdf_bytes = b"%PDF-1.0\n1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj 2 0 obj<</Type/Pages/Kids[3 0 R]/Count 1>>endobj 3 0 obj<</Type/Page/MediaBox[0 0 3 3]>>endobj\nxref\n0 4\n0000000000 65535 f\n0000000010 00000 n\n0000000060 00000 n\n0000000111 00000 n\ntrailer<</Size 4/Root 1 0 R>>\nstartxref\n178\n%%EOF";

    info!("uploading file...");
    let file_handle = client
        .create_file(dummy_pdf_bytes.to_vec())
        .display_name("test_doc.pdf")
        .with_mime_type("application/pdf".parse().unwrap())
        .upload()
        .await?;

    info!(name = file_handle.name(), "file uploaded");

    // Use the file handle in generation request
    // This avoids re-encoding and re-sending the file content
    info!("generating content with file reference...");
    let response = client
        .generate_content()
        .with_user_message_and_file("Describe this document", &file_handle)?
        .execute()
        .await?;

    info!(response = response.text(), "received response");

    // Cleanup
    info!("cleaning up by deleting the file");
    match file_handle.delete().await {
        Ok(_) => info!("file deleted successfully"),
        Err((_, e)) => error!(error = %e, "failed to delete file"),
    }

    Ok(())
}
