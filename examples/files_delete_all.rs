//! Deletes all files associated with the API key.
use futures::stream::TryStreamExt;
use gemini_rust::{ClientError, Gemini};
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), ClientError> {
    tracing_subscriber::fmt::init();

    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
    let gemini = Gemini::new(&api_key)?;

    info!("fetching and deleting files");

    let any_files_found = AtomicBool::new(false);

    gemini
        .list_files(None)
        .try_for_each_concurrent(10, |file| {
            // Concurrently delete up to 10 files at a time
            any_files_found.store(true, Ordering::SeqCst);
            async move {
                info!(
                    display_name = file
                        .get_file_meta()
                        .display_name
                        .clone()
                        .unwrap_or_default(),
                    file_name = file.name(),
                    "deleting file"
                );

                match file.delete().await {
                    Ok(_) => info!("file deleted successfully"),
                    Err((_, e)) => error!(error = %e, "failed to delete file"),
                }
                Ok(())
            }
        })
        .await?;

    if !any_files_found.load(Ordering::SeqCst) {
        info!("no files found to delete");
    } else {
        info!("deletion process complete");
    }

    Ok(())
}
