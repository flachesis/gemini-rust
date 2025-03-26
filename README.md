# gemini-rust

A Rust client library for Google's Gemini 2.0 API.

## Features

- Complete implementation of the Gemini 2.0 API
- Support for system prompts, user prompts
- Tools and function calling (including Google Search)
- Streaming responses
- Async/await API

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
gemini-rust = "0.1.0"
```

## Usage

### Basic Usage

```rust
use gemini_rust::{Gemini, Message, Role, Content};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("GEMINI_API_KEY")?;
    let client = Gemini::new(&api_key);
    
    let response = client.generate_content()
        .with_system_prompt("You are a helpful assistant.")
        .with_user_message("Hello, how are you?")
        .execute()
        .await?;
    
    println!("Response: {}", response.text());
    
    Ok(())
}
```

### Using Google Search Tool

```rust
use gemini_rust::{Gemini, Tool};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("GEMINI_API_KEY")?;
    let client = Gemini::new(&api_key);
    
    // Create a Google Search tool
    let google_search_tool = Tool::google_search();
    
    let response = client.generate_content()
        .with_user_message("What is the current Google stock price?")
        .with_tool(google_search_tool)
        .execute()
        .await?;
    
    println!("Response: {}", response.text());
    
    Ok(())
}
```

## Documentation

For more examples and detailed documentation, see [docs.rs](https://docs.rs/gemini-rust).

## License

MIT