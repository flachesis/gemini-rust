# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.2.1] - 2025-08-29

### 🛠️ Bug Fixes

#### Content Structure Improvements
- **Fixed Content Serialization**: Resolved issues with Content structure serialization to match Gemini API requirements
  - Changed `Content.parts` from `Vec<Part>` to `Option<Vec<Part>>` to handle API responses where parts may be absent
  - Added `#[serde(rename_all = "camelCase")]` to `Content` struct for proper JSON field naming
  - Added `#[serde(rename_all = "camelCase")]` to `GenerateContentRequest` for consistent API formatting
  - Fixed `GenerationConfig` serialization with proper camelCase field naming

#### API Response Handling
- **Enhanced Response Parsing**: Improved handling of Gemini API responses with missing content parts
  - Updated `GenerationResponse.text()` method to safely handle `Option<Vec<Part>>`
  - Updated `GenerationResponse.function_calls()` method with proper option handling
  - Updated `GenerationResponse.thoughts()` and `GenerationResponse.all_text()` methods for safe access
  - Added support for missing parts in API responses (common with certain model configurations)

#### Example Updates
- **Fixed Example Code**: Updated examples to work with new Content structure
  - Updated `examples/advanced.rs` to use `Option<Vec<Part>>` when manually building content
  - Updated `examples/curl_equivalent.rs` with proper Content construction
  - Updated `examples/curl_google_search.rs` for compatibility
  - Improved `examples/simple.rs` with better token limits (`max_output_tokens: 1000`)

#### Additional Response Fields
- **Extended Response Model**: Added missing fields to `GenerationResponse`
  - Added `model_version: Option<String>` field for tracking model version information
  - Added `response_id: Option<String>` field for response identification
  - Enhanced `UsageMetadata` with proper field structure for token counting

### 🔧 Technical Improvements

#### Serialization Consistency
- **Unified camelCase Naming**: Ensured all API-facing structs use consistent camelCase field naming
  - Prevents JSON serialization mismatches with Gemini API
  - Improves reliability of API communication
  - Maintains backward compatibility in Rust code

#### Error Resilience
- **Robust Content Handling**: Improved handling of edge cases in API responses
  - Better support for responses with empty or missing content parts
  - Safer default values for optional fields
  - Reduced likelihood of deserialization failures

### 📋 Usage Impact

#### Breaking Changes
- **Content Construction**: Direct manipulation of `Content.parts` now requires `Option` wrapping
  ```rust
  // Old (no longer works)
  content.parts.push(part);

  // New (correct approach)
  content.parts = Some(vec![part]);
  ```

#### Migration Guide
- **Automatic Migration**: Most users won't need changes as the `Content::text()`, `Content::function_call()`, etc. helper methods handle the Option wrapping automatically
- **Direct Content Building**: Only users manually constructing Content structs need to wrap parts in `Some()`

### 🙏 Contributors

- **@flachesis** - Comprehensive Content structure refactoring and API compatibility improvements

### 📝 Notes

- This release improves compatibility with various Gemini API response formats
- No functional changes to public API methods - all builder patterns work unchanged
- Enhanced error resilience when processing API responses with missing content parts
- Better support for different model configurations that may return sparse content

## [1.2.0] - 2025-08-29

### ✨ Features

#### Batch Content Generation API
- **Asynchronous Batch Processing**: Complete implementation of Gemini API's batch content generation
  - Support for submitting multiple content generation requests in a single batch operation
  - Proper handling of asynchronous batch processing with batch tracking
  - Detailed batch status monitoring with request counts and state tracking
  - Full compliance with Google's batch API format including nested request structures

#### Enhanced API Structure
- **Batch Request Models**: New comprehensive type system for batch operations
  - `BatchGenerateContentRequest` with proper nested structure (`batch.input_config.requests.requests`)
  - `BatchConfig` for batch configuration with display names
  - `InputConfig` and `RequestsContainer` for structured request organization
  - `BatchRequestItem` with metadata support for individual requests
  - `RequestMetadata` for request identification and tracking

#### Improved Response Handling
- **Batch Response Processing**: Detailed batch operation response handling
  - `BatchGenerateContentResponse` with batch creation confirmation
  - `BatchMetadata` including creation/update timestamps and model information
  - `BatchStats` with comprehensive request counting (pending, completed, failed)
  - Proper state tracking for batch operations (`BATCH_STATE_PENDING`, etc.)

#### Public API Enhancements
- **Extended Type Exports**: Additional types now available from crate root
  - `ContentBuilder` now publicly exported for advanced usage patterns
  - `GenerateContentRequest` accessible for custom request building
  - All batch-related types exported for external batch management
  - Enhanced builder pattern accessibility

### 🛠️ Technical Improvements

#### Dependency Management
- **Development Dependencies Optimization**: Moved `tokio` to dev-dependencies
  - Reduced production bundle size by moving tokio to development-only dependencies
  - Maintained full async functionality while optimizing dependency tree
  - Better separation of concerns between runtime and library dependencies

#### Code Organization
- **Enhanced Builder Architecture**: Improved batch builder implementation
  - Automatic metadata generation for batch requests
  - Streamlined batch creation with fluent API
  - Better error handling and validation for batch operations

### 📋 Usage Examples

#### Batch Content Generation
```rust
use gemini_rust::{Gemini, Message};

let client = Gemini::new(api_key);

// Create individual requests
let request1 = client
    .generate_content()
    .with_message(Message::user("What is the meaning of life?"))
    .build();

let request2 = client
    .generate_content()
    .with_message(Message::user("What is the best programming language?"))
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
```

#### Advanced Request Building
```rust
use gemini_rust::{ContentBuilder, GenerateContentRequest};

// Direct access to ContentBuilder for advanced patterns
let mut builder: ContentBuilder = client.generate_content();
let request: GenerateContentRequest = builder
    .with_user_message("Custom request")
    .build();
```

### 🙏 Contributors

- **@npatsakula** - Implemented basic batch API foundation ([#8](https://github.com/flachesis/gemini-rust/pull/8))
- **@brekkylab** - Optimized dependency management ([#7](https://github.com/flachesis/gemini-rust/pull/7))
- **@flachesis** - Enhanced batch API with detailed request/response structures

### 🔄 Breaking Changes

None. This release maintains full backward compatibility with v1.1.0.

### 📝 Notes

- Batch operations are asynchronous and require polling for completion status
- Batch API follows Google's official format specification exactly
- Enhanced type safety with comprehensive batch operation modeling
- Improved error handling for batch-specific scenarios

## [1.1.0] - 2025-07-21

### ✨ Features

#### Public API Enhancements
- **`Blob` Type Export**: The `Blob` struct is now publicly exported from the crate root
  - Enables direct usage of `Blob` for inline data handling without importing from internal modules
  - Improves ergonomics for multimodal applications working with images and binary data
  - Maintains all existing functionality including `new()` constructor and base64 encoding support

### 🙏 Contributors

- **@anandkapdi** - Made `Blob` struct publicly accessible ([#6](https://github.com/flachesis/gemini-rust/pull/6))

### 📋 Usage Examples

#### Direct Blob Usage
```rust
use gemini_rust::{Gemini, Blob};

// Now you can use Blob directly from the crate root
let blob = Blob::new("image/jpeg", base64_encoded_data);
let response = client
    .generate_content()
    .with_user_message("What's in this image?")
    .with_inline_data(blob)
    .execute()
    .await?;
```

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
