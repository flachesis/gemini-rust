use gemini_rust::{Gemini, TaskType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {    
    let api_key = std::env::var("GEMINI_API_KEY")?;

    // Create client with the default model (gemini-2.0-flash)
    let client = Gemini::with_model(api_key, "models/text-embedding-004".to_string());

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