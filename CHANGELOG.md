# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-07-12

### 🎉 Initial Stable Release

This marks the first stable release of `gemini-rust`, a comprehensive Rust client library for Google's Gemini 2.0 API. This release consolidates all the features developed during the pre-1.0 phase into a stable, production-ready library.

### ✨ Features

#### Core API Support
- **Content Generation**: Complete implementation of Gemini 2.0 content generation API
  - Support for system prompts and user messages
  - Multi-turn conversations with conversation history
  - Configurable generation parameters (temperature, max tokens, etc.)
  - Safety settings and content filtering
- **Streaming Responses**: Real-time streaming of generated content using async streams
- **Text Embeddings**: Full support for text embedding generation
  - Multiple task types: `RetrievalDocument`, `RetrievalQuery`, `SemanticSimilarity`, `Classification`, `Clustering`
  - Batch embedding support for processing multiple texts efficiently
  - Support for `text-embedding-004` and other embedding models

#### Advanced Features
- **Function Calling & Tools**: Complete tools and function calling implementation
  - Custom function declarations with JSON schema validation
  - Google Search tool integration for real-time web search capabilities
  - Function response handling and multi-turn tool conversations
  - Support for multiple tools per request
- **Thinking Mode**: Support for Gemini 2.5 series thinking capabilities
  - Dynamic thinking (model determines thinking budget automatically)
  - Fixed thinking budget with configurable token limits
  - Access to thinking process summaries
  - Thought inclusion in responses for transparency
- **Multimodal Support**: Support for images and binary data
  - Inline data support with base64 encoding
  - Multiple MIME type support
  - Blob handling for various media types

#### Technical Excellence
- **Async/Await**: Full async support built on `tokio` runtime
- **Type Safety**: Comprehensive type definitions with `serde` serialization
- **Error Handling**: Robust error handling with detailed error types using `thiserror`
- **Builder Pattern**: Fluent, ergonomic API design for easy usage
- **HTTP/2 Support**: Modern HTTP features with `reqwest` client
- **Configurable Endpoints**: Support for custom base URLs and API endpoints

### 🏗️ Architecture

#### Core Components
- **`Gemini` Client**: Main client struct with model configuration and API key management
- **`ContentBuilder`**: Fluent API for building content generation requests
- **`EmbedBuilder`**: Specialized builder for embedding requests
- **Type System**: Complete type definitions matching Gemini API specifications
- **Error Types**: Comprehensive error handling covering HTTP, JSON, and API errors

#### Models & Types
- **`GenerationResponse`**: Complete response parsing with candidates, safety ratings, and metadata
- **`Content` & `Part`**: Flexible content representation supporting text, function calls, and media
- **`Tool` & `FunctionDeclaration`**: Full function calling type system
- **`ThinkingConfig`**: Configuration for Gemini 2.5 thinking capabilities
- **`UsageMetadata`**: Token usage tracking and billing information

### 📚 Examples & Documentation

#### Comprehensive Examples
- **`simple.rs`**: Basic text generation with system prompts
- **`streaming.rs`**: Real-time streaming response handling
- **`embedding.rs`**: Text embedding generation
- **`batch_embedding.rs`**: Efficient batch processing of embeddings
- **`google_search.rs`**: Google Search tool integration
- **`tools.rs`**: Custom function calling examples
- **`thinking_basic.rs`** & **`thinking_advanced.rs`**: Gemini 2.5 thinking mode usage
- **`blob.rs`**: Image and binary data handling
- **`structured_response.rs`**: Structured output generation
- **`generation_config.rs`**: Advanced generation configuration
- **`custom_base_url.rs`**: Custom endpoint configuration

#### CURL Equivalents
- **`curl_equivalent.rs`**: Direct API comparison examples
- **`curl_google_search.rs`**: Google Search API equivalent
- **`curl_thinking_equivalent.rs`**: Thinking mode API equivalent

### 🔧 Dependencies

#### Production Dependencies
- **`reqwest ^0.12.15`**: HTTP client with JSON, streaming, and HTTP/2 support
- **`serde ^1.0`**: Serialization framework with derive support
- **`serde_json ^1.0`**: JSON serialization support
- **`tokio ^1.28`**: Async runtime with full feature set
- **`thiserror ^2.0.12`**: Error handling and derivation
- **`url ^2.4`**: URL parsing and manipulation
- **`async-trait ^0.1`**: Async trait support
- **`futures ^0.3.1`** & **`futures-util ^0.3`**: Async utilities and stream handling
- **`base64 0.22.1`**: Base64 encoding for binary data

### 🛡️ API Compatibility

#### Supported Models
- **Gemini 2.5 Flash**: Default model with thinking capabilities
- **Gemini 2.5 Pro**: Advanced model with enhanced thinking features
- **Gemini 2.0 Flash**: Fast generation model
- **Text Embedding 004**: Latest embedding model
- **Custom Models**: Support for any Gemini API compatible model

#### API Endpoints
- **Generate Content**: `/v1beta/models/{model}:generateContent`
- **Stream Generate Content**: `/v1beta/models/{model}:streamGenerateContent`
- **Embed Content**: `/v1beta/models/{model}:embedContent`
- **Batch Embed Content**: `/v1beta/models/{model}:batchEmbedContents`
- **Custom Base URLs**: Configurable endpoint support

### 🔒 Security & Safety

- **API Key Management**: Secure API key handling through environment variables
- **Content Safety**: Built-in safety rating parsing and handling
- **Error Resilience**: Comprehensive error handling for network and API issues
- **Input Validation**: Type-safe request building preventing malformed requests

### 📋 Usage Examples

#### Basic Text Generation
```rust
use gemini_rust::Gemini;

let client = Gemini::new(api_key);
let response = client
    .generate_content()
    .with_system_prompt("You are a helpful assistant.")
    .with_user_message("Hello!")
    .execute()
    .await?;
```

#### Streaming Responses
```rust
let mut stream = client
    .generate_content()
    .with_user_message("Tell me a story")
    .execute_stream()
    .await?;

while let Some(chunk) = stream.next().await {
    print!("{}", chunk?.text());
}
```

#### Google Search Integration
```rust
use gemini_rust::Tool;

let response = client
    .generate_content()
    .with_user_message("What's the current weather?")
    .with_tool(Tool::google_search())
    .execute()
    .await?;
```

#### Text Embeddings
```rust
let response = client
    .embed_content()
    .with_text("Hello, world!")
    .with_task_type(TaskType::RetrievalDocument)
    .execute()
    .await?;
```

#### Thinking Mode (Gemini 2.5)
```rust
let client = Gemini::with_model(api_key, "models/gemini-2.5-pro");
let response = client
    .generate_content()
    .with_user_message("Explain quantum physics")
    .with_dynamic_thinking()
    .with_thoughts_included(true)
    .execute()
    .await?;
```

### 🚀 Performance

- **Async/Await**: Non-blocking I/O for high concurrency
- **HTTP/2**: Efficient connection reuse and multiplexing
- **Streaming**: Memory-efficient processing of large responses
- **Batch Processing**: Optimized batch embedding support
- **Connection Pooling**: Automatic connection management

### 📝 Documentation

- **Comprehensive README**: Detailed usage examples and API overview
- **Inline Documentation**: Complete rustdoc documentation for all public APIs
- **Example Collection**: 20+ examples covering all major features
- **Type Documentation**: Full documentation of all models and types

### 🔗 Links

- **Repository**: [https://github.com/flachesis/gemini-rust](https://github.com/flachesis/gemini-rust)
- **Documentation**: Available on docs.rs
- **Crates.io**: [https://crates.io/crates/gemini-rust](https://crates.io/crates/gemini-rust)
- **License**: MIT

---

**Note**: This 1.0.0 release represents a stable API that follows semantic versioning. Future releases will maintain backward compatibility within the 1.x series, with breaking changes reserved for major version increments.
