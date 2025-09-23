use futures_util::TryStreamExt;
use gemini_rust::Gemini;
use std::env;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt::init();
    // Get API key from environment variable
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY environment variable not set");

    // Create client
    let client = Gemini::new(api_key).expect("unable to create Gemini API client");

    // Simple streaming generation
    info!("starting streaming generation example");

    let mut stream = client
        .generate_content()
        .with_thinking_budget(0)
        .with_system_prompt("You are a helpful, creative assistant.")
        .with_user_message("Write a short story about a robot who learns to feel emotions.")
        .execute_stream()
        .await?;

    // pin!(stream);

    info!("streaming response chunks");
    let mut full_response = String::new();
    while let Some(chunk) = stream.try_next().await? {
        let chunk_text = chunk.text();
        full_response.push_str(&chunk_text);
        print!("{}", chunk_text);
        std::io::Write::flush(&mut std::io::stdout())?;
    }
    println!();
    info!(response = full_response, "streaming generation completed");

    // Multi-turn conversation
    info!("starting multi-turn conversation example");

    // First turn
    info!(
        question = "I'm planning a trip to Japan. What are the best times to visit?",
        "sending first turn"
    );
    let response1 = client
        .generate_content()
        .with_system_prompt("You are a helpful travel assistant.")
        .with_user_message("I'm planning a trip to Japan. What are the best times to visit?")
        .execute()
        .await?;

    info!(
        question = "I'm planning a trip to Japan. What are the best times to visit?",
        response = response1.text(),
        "first turn completed"
    );

    // Second turn (continuing the conversation)
    info!(
        question = "What about cherry blossom season? When exactly does that happen?",
        "sending second turn"
    );
    let response2 = client
        .generate_content()
        .with_system_prompt("You are a helpful travel assistant.")
        .with_user_message("I'm planning a trip to Japan. What are the best times to visit?")
        .with_model_message(response1.text())
        .with_user_message("What about cherry blossom season? When exactly does that happen?")
        .execute()
        .await?;

    info!(
        question = "What about cherry blossom season? When exactly does that happen?",
        response = response2.text(),
        "second turn completed"
    );

    // Third turn (continuing the conversation)
    info!(
        question = "What are some must-visit places in Tokyo?",
        "sending third turn"
    );
    let response3 = client
        .generate_content()
        .with_system_prompt("You are a helpful travel assistant.")
        .with_user_message("I'm planning a trip to Japan. What are the best times to visit?")
        .with_model_message(response1.text())
        .with_user_message("What about cherry blossom season? When exactly does that happen?")
        .with_model_message(response2.text())
        .with_user_message("What are some must-visit places in Tokyo?")
        .execute()
        .await?;

    info!(
        question = "What are some must-visit places in Tokyo?",
        response = response3.text(),
        "third turn completed"
    );

    Ok(())
}
