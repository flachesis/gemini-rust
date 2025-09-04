# gemini-rust

A comprehensive Rust client library for Google's Gemini 2.5 API.

[![Crates.io](https://img.shields.io/crates/v/gemini-rust.svg)](https://crates.io/crates/gemini-rust)
[![Documentation](https://docs.rs/gemini-rust/badge.svg)](https://docs.rs/gemini-rust)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## ✨ Features

- **🚀 Complete Gemini 2.5 API Implementation** - Full support for all Gemini API endpoints
- **🛠️ Function Calling & Tools** - Custom functions and Google Search integration
- **📦 Batch Processing** - Efficient batch content generation and embedding
- **🔄 Streaming Responses** - Real-time streaming of generated content
- **🧠 Thinking Mode** - Support for Gemini 2.5 thinking capabilities
- **🎨 Image Generation** - Text-to-image generation and image editing capabilities
- **🎤 Speech Generation** - Text-to-speech with single and multi-speaker support
- **🖼️ Multimodal Support** - Images and binary data processing
- **📊 Text Embeddings** - Advanced embedding generation with multiple task types
- **⚙️ Highly Configurable** - Custom models, endpoints, and generation parameters
- **🔒 Type Safe** - Comprehensive type definitions with full `serde` support
- **⚡ Async/Await** - Built on `tokio` for high-performance async operations

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
gemini-rust = "1.3.1"
```

## 🚀 Quick Start

### Basic Content Generation

```rust
use gemini_rust::Gemini;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("GEMINI_API_KEY")?;
    let client = Gemini::new(api_key)?;

    let response = client
        .generate_content()
        .with_system_prompt("You are a helpful assistant.")
        .with_user_message("Hello, how are you?")
        .execute()
        .await?;

    println!("Response: {}", response.text());
    Ok(())
}
```

### Streaming Responses

```rust
use gemini_rust::Gemini;
use futures::TryStreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Gemini::new(std::env::var("GEMINI_API_KEY")?)?;

    let mut stream = client
        .generate_content()
        .with_user_message("Tell me a story about programming")
        .execute_stream()
        .await?;

    while let Some(chunk) = stream.try_next().await? {
        print!("{}", chunk.text());
    }
    Ok(())
}
```

## 🛠️ Advanced Features

### Function Calling

```rust
use gemini_rust::{Gemini, FunctionDeclaration, FunctionParameters, PropertyDetails};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Gemini::new(std::env::var("GEMINI_API_KEY")?)?;

    // Define a custom function
    let weather_function = FunctionDeclaration::new(
        "get_weather",
        "Get the current weather for a location",
        FunctionParameters::object()
            .with_property(
                "location",
                PropertyDetails::string("The city and state, e.g., San Francisco, CA"),
                true,
            )
    );

    let response = client
        .generate_content()
        .with_user_message("What's the weather like in Tokyo?")
        .with_function(weather_function)
        .execute()
        .await?;

    // Handle function calls
    if let Some(function_call) = response.function_calls().first() {
        println!("Function: {}", function_call.name);
        println!("Args: {}", function_call.args);
    }
    Ok(())
}
```

### Image Generation

```rust
use gemini_rust::Gemini;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Gemini::with_model(
        std::env::var("GEMINI_API_KEY")?,
        "models/gemini-2.5-flash-image-preview".to_string()
    )?;

    let response = client
        .generate_content()
        .with_user_message(
            "Create a photorealistic image of a cute robot sitting in a garden, \
             surrounded by colorful flowers. The robot should have a friendly \
             expression and be made of polished metal."
        )
        .execute()
        .await?;

    // Save generated images
    for candidate in response.candidates.iter() {
        if let Some(parts) = &candidate.content.parts {
            for part in parts.iter() {
                if let gemini_rust::Part::InlineData { inline_data } = part {
                    let image_bytes = BASE64.decode(&inline_data.data)?;
                    fs::write("generated_image.png", image_bytes)?;
                    println!("Image saved as generated_image.png");
                }
            }
        }
    }
    Ok(())
}
```

### Speech Generation

```rust
use gemini_rust::{Gemini, GenerationConfig, SpeechConfig, VoiceConfig, PrebuiltVoiceConfig};
use base64::{Engine as _, engine::general_purpose};
use std::fs::File;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Gemini::with_model(
        std::env::var("GEMINI_API_KEY")?,
        "models/gemini-2.5-flash-preview-tts".to_string()
    )?;

    let generation_config = GenerationConfig {
        response_modalities: Some(vec!["AUDIO".to_string()]),
        speech_config: Some(SpeechConfig {
            voice_config: Some(VoiceConfig {
                prebuilt_voice_config: Some(PrebuiltVoiceConfig {
                    voice_name: "Puck".to_string(),
                }),
            }),
            multi_speaker_voice_config: None,
        }),
        ..Default::default()
    };

    let response = client
        .generate_content()
        .with_user_message("Hello! Welcome to Gemini text-to-speech.")
        .with_generation_config(generation_config)
        .execute()
        .await?;

    // Save generated audio as PCM format
    for candidate in response.candidates.iter() {
        if let Some(parts) = &candidate.content.parts {
            for part in parts.iter() {
                if let gemini_rust::Part::InlineData { inline_data } = part {
                    if inline_data.mime_type.starts_with("audio/") {
                        let audio_bytes = general_purpose::STANDARD.decode(&inline_data.data)?;
                        let mut file = File::create("speech_output.pcm")?;
                        file.write_all(&audio_bytes)?;
                        println!("Audio saved as speech_output.pcm");
                        println!("Convert to WAV using: ffmpeg -f s16le -ar 24000 -ac 1 -i speech_output.pcm speech_output.wav");
                    }
                }
            }
        }
    }
    Ok(())
}
```

### Google Search Tool

```rust
use gemini_rust::{Gemini, Tool};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Gemini::new(std::env::var("GEMINI_API_KEY")?)?;

    let response = client
        .generate_content()
        .with_user_message("What's the latest news about Rust programming language?")
        .with_tool(Tool::google_search())
        .execute()
        .await?;

    println!("Response: {}", response.text());
    Ok(())
}
```

### Thinking Mode (Gemini 2.5)

```rust
use gemini_rust::Gemini;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use Gemini 2.5 Pro for advanced thinking capabilities
    let client = Gemini::pro(std::env::var("GEMINI_API_KEY")?)?;

    let response = client
        .generate_content()
        .with_user_message("Explain quantum computing in simple terms")
        .with_dynamic_thinking()  // Let model decide thinking budget
        .with_thoughts_included(true)  // Include thinking process
        .execute()
        .await?;

    // Access thinking summaries
    for thought in response.thoughts() {
        println!("Thought: {}", thought);
    }

    println!("Response: {}", response.text());
    Ok(())
}
```

### Text Embeddings

```rust
use gemini_rust::{Gemini, Model, TaskType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Gemini::with_model(
        std::env::var("GEMINI_API_KEY")?,
        Model::TextEmbedding004
    )?;

    let response = client
        .embed_content()
        .with_text("Hello, this is my text to embed")
        .with_task_type(TaskType::RetrievalDocument)
        .execute()
        .await?;

    println!("Embedding dimensions: {}", response.embedding.values.len());
    Ok(())
}
```

### Batch Processing

```rust
use gemini_rust::{Gemini, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Gemini::new(std::env::var("GEMINI_API_KEY")?)?;

    // Create multiple requests
    let request1 = client
        .generate_content()
        .with_user_message("What is the meaning of life?")
        .build();

    let request2 = client
        .generate_content()
        .with_user_message("What is the best programming language?")
        .build();

    // Submit batch request
    let batch_response = client
        .batch_generate_content_sync()
        .with_request(request1)
        .with_request(request2)
        .execute()
        .await?;

    println!("Batch ID: {}", batch_response.name);
    println!("State: {}", batch_response.metadata.state);
    Ok(())
}
```

### Image Processing

```rust
use gemini_rust::{Gemini, Blob};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Gemini::new(std::env::var("GEMINI_API_KEY")?)?;

    // Load and encode image as base64
    let image_data = std::fs::read("path/to/image.jpg")?;
    let base64_image = base64::encode(&image_data);

    let blob = Blob::new("image/jpeg", base64_image);

    let response = client
        .generate_content()
        .with_user_message("What's in this image?")
        .with_inline_data("image/jpeg", base64_image)
        .execute()
        .await?;

    println!("Response: {}", response.text());
    Ok(())
}
```

### Generation Configuration

```rust
use gemini_rust::{Gemini, GenerationConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Gemini::new(std::env::var("GEMINI_API_KEY")?)?;

    let response = client
        .generate_content()
        .with_user_message("Write a creative story")
        .with_generation_config(GenerationConfig {
            temperature: Some(0.9),
            max_output_tokens: Some(1000),
            top_p: Some(0.8),
            top_k: Some(40),
            stop_sequences: Some(vec!["END".to_string()]),
            ..Default::default()
        })
        .execute()
        .await?;

    println!("Response: {}", response.text());
    Ok(())
}
```

## 🔧 Configuration

### Custom Models

```rust
use gemini_rust::Gemini;

// Use Gemini 2.5 Flash (default)
let client = Gemini::new(api_key)?;

// Use Gemini 2.5 Pro for advanced tasks
let client = Gemini::pro(api_key)?;

// Use specific model
let client = Gemini::with_model(api_key, "models/gemini-1.5-pro".to_string())?;
```

### Custom Base URL

```rust
use gemini_rust::Gemini;

// Custom endpoint
let client = Gemini::with_base_url(
    api_key,
    "https://custom-api.example.com/v1/".parse()?
)?;

// Custom model and endpoint
let client = Gemini::with_model_and_base_url(
    api_key,
    "models/gemini-pro".to_string(),
    "https://custom-api.example.com/v1/".parse()?
)?;
```

## 📚 Examples

The repository includes comprehensive examples:

| Example | Description |
|---------|-------------|
| [`simple.rs`](examples/simple.rs) | Basic text generation and function calling |
| [`advanced.rs`](examples/advanced.rs) | Advanced content generation with all parameters |
| [`streaming.rs`](examples/streaming.rs) | Real-time streaming responses |
| [`tools.rs`](examples/tools.rs) | Custom function declarations |
| [`google_search.rs`](examples/google_search.rs) | Google Search integration |
| [`google_search_with_functions.rs`](examples/google_search_with_functions.rs) | Google Search with custom functions |
| [`thinking_basic.rs`](examples/thinking_basic.rs) | Gemini 2.5 thinking mode (basic) |
| [`thinking_advanced.rs`](examples/thinking_advanced.rs) | Gemini 2.5 thinking mode (advanced) |
| [`batch_generate.rs`](examples/batch_generate.rs) | Batch content generation |
| [`batch_cancel.rs`](examples/batch_cancel.rs) | Batch operation cancellation |
| [`batch_delete.rs`](examples/batch_delete.rs) | Batch operation deletion |
| [`batch_list.rs`](examples/batch_list.rs) | Batch operation listing with streaming |
| [`batch_embedding.rs`](examples/batch_embedding.rs) | Batch text embedding generation |
| [`embedding.rs`](examples/embedding.rs) | Text embedding generation |
| [`error_handling.rs`](examples/error_handling.rs) | Error handling examples |
| [`blob.rs`](examples/blob.rs) | Image and binary data processing |
| [`files_delete_all.rs`](examples/files_delete_all.rs) | Delete all files |
| [`files_lifecycle.rs`](examples/files_lifecycle.rs) | Full file lifecycle |
| [`simple_image_generation.rs`](examples/simple_image_generation.rs) | Basic text-to-image generation |
| [`image_generation.rs`](examples/image_generation.rs) | Advanced image generation examples |
| [`image_editing.rs`](examples/image_editing.rs) | Image editing with text prompts |
| [`simple_speech_generation.rs`](examples/simple_speech_generation.rs) | Basic text-to-speech generation |
| [`multi_speaker_tts.rs`](examples/multi_speaker_tts.rs) | Multi-speaker text-to-speech dialogue |
| [`structured_response.rs`](examples/structured_response.rs) | Structured JSON output |
| [`generation_config.rs`](examples/generation_config.rs) | Custom generation parameters |
| [`custom_base_url.rs`](examples/custom_base_url.rs) | Using a custom API endpoint |
| [`curl_equivalent.rs`](examples/curl_equivalent.rs) | Equivalent cURL commands for API calls |

Run an example:

```bash
GEMINI_API_KEY="your-api-key" cargo run --example simple
```

## 🔑 API Key Setup

Get your API key from [Google AI Studio](https://aistudio.google.com/apikey) and set it as an environment variable:

```bash
export GEMINI_API_KEY="your-api-key-here"
```

## 🚦 Supported Models

- **Gemini 2.5 Flash** - Fast, efficient model (default) - `Model::Gemini25Flash`
- **Gemini 2.5 Flash Lite** - Lightweight model - `Model::Gemini25FlashLite`
- **Gemini 2.5 Pro** - Advanced model with thinking capabilities - `Model::Gemini25Pro`
- **Text Embedding 004** - Latest embedding model - `Model::TextEmbedding004`
- **Custom models** - Use `Model::Custom(String)` or string literals for other models

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Google for providing the Gemini API
- The Rust community for excellent async and HTTP libraries
- All contributors who have helped improve this library
