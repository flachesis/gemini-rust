use gemini_rust::{Gemini, Model, TaskType};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt::init();

    let api_key = std::env::var("GEMINI_API_KEY")?;

    // Create client with the default model (gemini-2.0-flash)
    let client = Gemini::with_model(api_key, Model::TextEmbedding004)
        .expect("unable to create Gemini API client");

    info!("sending embedding request to gemini api");

    // Simple text embedding
    let response = client
        .embed_content()
        .with_text("Hello")
        .with_task_type(TaskType::RetrievalDocument)
        .execute()
        .await?;

    info!(
        embedding_length = response.embedding.values.len(),
        first_values = ?&response.embedding.values[..5.min(response.embedding.values.len())],
        "embedding completed"
    );

    Ok(())
}
