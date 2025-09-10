//! Deletes all files associated with the API key.
use futures::stream::TryStreamExt;
use gemini_rust::{ClientError, Gemini};
use std::sync::atomic::{AtomicBool, Ordering};

#[tokio::main]
async fn main() -> Result<(), ClientError> {
    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
    let gemini = Gemini::new(&api_key)?;

    println!("Fetching and deleting files...");

    let any_files_found = AtomicBool::new(false);

    gemini
        .list_files(None)
        .try_for_each_concurrent(10, |file| {
            // Concurrently delete up to 10 files at a time
            any_files_found.store(true, Ordering::SeqCst);
            async move {
                println!(
                    "Deleting {} ({})",
                    file.get_file_meta()
                        .display_name
                        .clone()
                        .unwrap_or_default(),
                    file.name()
                );

                match file.delete().await {
                    Ok(_) => println!("Success."),
                    Err((_, e)) => println!("Failed: {}", e),
                }
                Ok(())
            }
        })
        .await?;

    if !any_files_found.load(Ordering::SeqCst) {
        println!("No files found to delete.");
    } else {
        println!("Deletion process complete.");
    }

    Ok(())
}
