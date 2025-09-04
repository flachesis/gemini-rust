use futures_util::TryStreamExt;
use gemini_rust::Gemini;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key from environment variable
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY environment variable not set");

    // Create client
    let client = Gemini::new(api_key).expect("unable to create Gemini API client");

    // Simple streaming generation
    println!("--- Streaming generation ---");

    let mut stream = client
        .generate_content()
        .with_system_prompt("You are a helpful, creative assistant.")
        .with_user_message("Write a short story about a robot who learns to feel emotions.")
        .execute_stream()
        .await?;

    // pin!(stream);

    print!("Streaming response: ");
    while let Some(chunk) = stream.try_next().await? {
        print!("{}", chunk.text());
        std::io::Write::flush(&mut std::io::stdout())?;
    }
    println!("\n");

    // Multi-turn conversation
    println!("--- Multi-turn conversation ---");

    // First turn
    let response1 = client
        .generate_content()
        .with_system_prompt("You are a helpful travel assistant.")
        .with_user_message("I'm planning a trip to Japan. What are the best times to visit?")
        .execute()
        .await?;

    println!("User: I'm planning a trip to Japan. What are the best times to visit?");
    println!("Assistant: {}\n", response1.text());

    // Second turn (continuing the conversation)
    let response2 = client
        .generate_content()
        .with_system_prompt("You are a helpful travel assistant.")
        .with_user_message("I'm planning a trip to Japan. What are the best times to visit?")
        .with_model_message(response1.text())
        .with_user_message("What about cherry blossom season? When exactly does that happen?")
        .execute()
        .await?;

    println!("User: What about cherry blossom season? When exactly does that happen?");
    println!("Assistant: {}\n", response2.text());

    // Third turn (continuing the conversation)
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

    println!("User: What are some must-visit places in Tokyo?");
    println!("Assistant: {}", response3.text());

    Ok(())
}
