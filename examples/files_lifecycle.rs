//! An example of uploading a file, downloading it, and verifying the content.
use futures::TryStreamExt;
use gemini_rust::{ClientError, Gemini};

const TEST_CONTENT: &str = "Hello, world! This is a test file for Gemini-Rust.";

#[tokio::main]
async fn main() -> Result<(), ClientError> {
    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
    let gemini = Gemini::new(&api_key)?;

    println!("Uploading a file with known content...");

    // 1. Upload a file
    let file_handle = gemini
        .create_file(TEST_CONTENT)
        .display_name("files-lifecycle-test.bin")
        .with_mime_type(mime::APPLICATION_OCTET_STREAM)
        .upload()
        .await?;

    println!("File uploaded successfully: {}", file_handle.name());

    // 2. Find remote file
    let files = gemini.list_files(None).try_collect::<Vec<_>>().await?;
    let remote_file = files
        .iter()
        .find(|file| file.name() == file_handle.name())
        .expect("uploaded file should be present");

    assert!(
        remote_file.download().await.is_err(),
        "Download of user-uploaded files is not supported by the Gemini API; only generated files can be downloaded"
    );

    // 3. Assert that the byte size is identical
    println!("\nVerifying file content...");
    assert_eq!(
        remote_file.get_file_meta().size_bytes.unwrap(),
        TEST_CONTENT.len() as i64,
        "File size does not match expected size!"
    );

    // 4. Delete the file
    println!("\nCleaning up by deleting the file...");
    match file_handle.delete().await {
        Ok(_) => println!("File deleted successfully."),
        Err((_, e)) => eprintln!("Failed to delete file: {}", e),
    }

    Ok(())
}
