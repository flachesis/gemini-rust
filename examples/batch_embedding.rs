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

    info!("sending batch embedding request to gemini api");

    // Simple text embedding
    let response = client
        .embed_content()
        .with_chunks(vec!["Hello", "World", "Test embedding 3"])
        .with_task_type(TaskType::RetrievalDocument)
        .execute_batch()
        .await?;

    info!(
        embeddings_count = response.embeddings.len(),
        "batch embedding completed"
    );

    for (i, e) in response.embeddings.iter().enumerate() {
        info!(
            index = i,
            embedding_length = e.values.len(),
            first_values = ?&e.values[..5.min(e.values.len())],
            "embedding result"
        );
    }

    Ok(())
}
