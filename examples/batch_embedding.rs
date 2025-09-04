use gemini_rust::{Gemini, Model, TaskType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("GEMINI_API_KEY")?;

    // Create client with the default model (gemini-2.0-flash)
    let client = Gemini::with_model(api_key, Model::TextEmbedding004)
        .expect("unable to create Gemini API client");

    println!("Sending batch embedding request to Gemini API...");

    // Simple text embedding
    let response = client
        .embed_content()
        .with_chunks(vec!["Hello", "World", "Test embedding 3"])
        .with_task_type(TaskType::RetrievalDocument)
        .execute_batch()
        .await?;

    println!("Response: ");
    for (i, e) in response.embeddings.iter().enumerate() {
        println!("|{}|: {:?}\n", i, e.values);
    }

    Ok(())
}
