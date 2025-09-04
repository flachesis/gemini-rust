use std::process::ExitCode;

use display_error_chain::DisplayErrorChain;
use gemini_rust::{ClientError, Gemini, Model, TaskType};

async fn do_main(api_key: &str) -> Result<(), ClientError> {
    let client = Gemini::with_model(api_key, Model::TextEmbedding004)
        .expect("unable to create Gemini API client");

    println!("Sending embedding request to Gemini API...");

    // Simple text embedding
    let response = client
        .embed_content()
        .with_text("Hello")
        .with_task_type(TaskType::RetrievalDocument)
        .execute()
        .await?;

    println!("Response: {:?}", response.embedding.values);

    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    let api_key = std::env::var("GEMINI_API_KEY").expect("no gemini api key provided");

    if let Err(err) = do_main(&api_key).await {
        let formated = DisplayErrorChain::new(err).to_string();
        eprintln!("{formated}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
