# gemini-rust

A comprehensive Rust client library for Google's Gemini API.

[![Crates.io](https://img.shields.io/crates/v/gemini-rust.svg)](https://crates.io/crates/gemini-rust)
[![Documentation](https://docs.rs/gemini-rust/badge.svg)](https://docs.rs/gemini-rust)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## ✨ Features

- **🤝 Interactions API (Recommended)** - Unified interface for models and agents with server-side state, observable execution steps, and background execution
- **🤖 Managed Agents** - Built-in Deep Research and Antigravity agents with sandbox environments
- **🔄 Background Execution** - Long-running interactions with polling and webhook callbacks
- **🚀 Complete Gemini API Implementation** - Full support for all Gemini API endpoints
- **🛠️ Function Calling & Tools** - Custom functions, Google Search, and Google Maps integration with OpenAPI schema support
- **🗺️ Google Maps Grounding** - Location-aware responses with Google Maps data and widget support
- **📦 Batch Processing** - Efficient batch content generation and embedding
- **💾 Content Caching** - Cache system instructions and conversation history for cost optimization
- **🔄 Streaming Responses** - Real-time streaming of generated content via SSE step lifecycle events
- **🧠 Thinking Mode** - Support for Gemini 2.5+ thinking capabilities
- **🚀 Gemini 3 Pro** - Code execution, advanced thinking levels, and media resolution control
- **🔢 Token Count API** - Pre-calculate token usage for cost optimization
- **🎯 Safety Settings** - Customize content moderation and safety filters
- **📁 File Handles** - Efficient file reference without re-encoding large files
- **🍌 Image Generation** - Text-to-image generation and image editing capabilities
- **🎤 Speech Generation** - Text-to-speech with single and multi-speaker support
- **🖼️ Multimodal Support** - Images and binary data processing
- **📊 Text Embeddings** - Advanced embedding generation with multiple task types
- **🔍 File Search** - Retrieval Augmented Generation (RAG) with semantic document search
- **⚙️ Highly Configurable** - Custom models, endpoints, and generation parameters with HTTP client builder
- **🔒 Type Safe** - Comprehensive type definitions with full `serde` support
- **⚡ Async/Await** - Built on `tokio` for high-performance async operations
- **🔍 Comprehensive Tracing** - Built-in structured logging and telemetry with `tracing` for observability

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
gemini-rust = "1.7.1"
```

## 🚀 Quick Start

### Interactions API (Recommended)

The Interactions API is the simplest and best way to use Gemini models and agents. It provides server-side state management, observable execution steps, background execution, and unified support for models and agents.

```rust,no_run
use gemini_rust::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("GEMINI_API_KEY")?;
    let client = Gemini::new(api_key)?;

    // Basic text generation
    let interaction = client
        .create_interaction()
        .with_model("gemini-2.5-flash")
        .with_text("Hello! What is AI?")
        .execute()
        .await?;

    println!("{}", interaction.output_text());

    // Multi-turn with server-side state
    let interaction2 = client
        .create_interaction()
        .with_model("gemini-2.5-flash")
        .with_text("Give me 3 examples")
        .with_previous_interaction(interaction.id().unwrap())
        .execute()
        .await?;

    println!("{}", interaction2.output_text());

    Ok(())
}
```

Key features:
- **Server-side state** — `previous_interaction_id` for multi-turn conversations
- **Background execution** — `.with_background()` + polling
- **Managed agents** — Deep Research, Antigravity
- **Observable steps** — thoughts, function calls, tool usage as typed steps
- **SSE streaming** — step lifecycle events (start/delta/stop)
- **Structured output** — JSON schema via `.with_json_schema()`

See the `interaction_*.rs` examples for complete coverage of every feature.

### Legacy generateContent API

The original `generateContent` API is still available but deprecated. New code should use the Interactions API.

## 🔁 Migration Guide: generateContent → Interactions API

### Paradigm Shift

The Interactions API introduces a fundamentally different request/response model:

```
generateContent (Legacy)                  Interactions API (Recommended)

Contents (messages)  →  Candidates        Interaction (input)  →  Steps (typed actions)
```

| Aspect | generateContent (Legacy) | Interactions API (Recommended) |
|---|---|---|
| **Request** | `contents: Vec<Content>` (role + parts) | `input: string \| Content[] \| Step[]` |
| **Response** | `candidates: Vec<Candidate>` (parts) | `steps: Vec<Step>` (typed enum) |
| **Multi-turn state** | Client manages full history | Server-side `previous_interaction_id` |
| **Streaming** | SSE chunks of `GenerationResponse` | SSE step lifecycle events (`step.start/delta/stop`) |
| **Function calling** | `Part::FunctionCall` / `Part::FunctionResponse` | `Step::FunctionCall` / `Step::FunctionResult` |
| **Background execution** | Not supported | `background=true` + polling or webhook |
| **Managed agents** | Not supported | Deep Research, Antigravity |
| **Sandbox environments** | Not supported | `environment: "remote"` |
| **Service tiers** | Not supported | `flex`, `standard`, `priority` |

### Side-by-Side Code Comparison

#### Basic Text Generation

```rust
// ── Legacy generateContent ──
let response = client.generate_content(
    GenerateContentRequest::builder()
        .model("gemini-2.5-flash")
        .contents(vec![Message::new_user("What is AI?")])
        .build()
).await?;
println!("{}", response.candidates[0].content.parts[0].text);

// ── Interactions API ──
let interaction = client.create_interaction()
    .with_model("gemini-2.5-flash")
    .with_text("What is AI?")
    .execute()
    .await?;
println!("{}", interaction.output_text());
```

#### Multi-Turn Conversation

```rust
// ── Legacy: manually resend entire history ──
let response2 = client.generate_content(
    GenerateContentRequest::builder()
        .model("gemini-2.5-flash")
        .contents(vec![
            Message::new_user("What is AI?"),
            Message::new_model(&response.candidates[0].content.parts[0].text),
            Message::new_user("Give me 3 examples"),
        ])
        .build()
).await?;

// ── Interactions: server-side state via previous_interaction_id ──
let interaction2 = client.create_interaction()
    .with_model("gemini-2.5-flash")
    .with_text("Give me 3 examples")
    .with_previous_interaction(interaction.id().unwrap())
    .execute()
    .await?;
```

#### Streaming

```rust
// ── Legacy: SSE chunks of GenerationResponse ──
use gemini_rust::prelude::*;
let mut stream = client.generate_content_stream(request).await?;
while let Some(chunk) = stream.next().await {
    let chunk = chunk?;
    if let Some(part) = chunk.candidates[0].content.parts.first() {
        print!("{}", part.text);
    }
}

// ── Interactions: SSE step lifecycle events ──
let mut stream = client.create_interaction_stream(
    client.create_interaction()
        .with_model("gemini-2.5-flash")
        .with_text("Write a haiku about Rust")
).await?;
while let Some(event) = stream.next().await {
    let event = event?;
    match event {
        InteractionEvent::StepStart(step) => { /* step began */ }
        InteractionEvent::StepDelta(delta) => { /* incremental text */ }
        InteractionEvent::StepStop(step) => { /* step finished */ }
        InteractionEvent::InteractionCompleted(i) => { /* done */ }
        _ => {}
    }
}
```

### Feature Availability

**New in Interactions API (not in generateContent):**
- Server-side conversation state (`previous_interaction_id`)
- Background execution with polling or webhooks
- Managed agents (Deep Research, Antigravity)
- Remote sandbox environments
- Observable execution steps (thoughts, tool calls, code execution)
- Service tiers (`flex`, `standard`, `priority`)
- Webhook callbacks

**Only in generateContent (not yet in Interactions API):**
- Batch API
- Explicit content caching
- Custom safety settings
- Video metadata (clipping, frame rate)
- Automatic function calling

### Type Mapping

| Legacy Type | Interactions Type | Notes |
|---|---|---|
| `Content` / `Part` | `InteractionContent` | Type-tagged polymorphic |
| `Candidate` | `Step::ModelOutput` | Filter `steps` for `ModelOutput` variant |
| `Part::Text` | `InteractionContent::Text` | |
| `Part::InlineData` | `InteractionContent::Image` / `Audio` / `Video` | Typed per media |
| `Part::FunctionCall` | `Step::FunctionCall` | Now a top-level step, not a Part |
| `Part::FunctionResponse` | `Step::FunctionResult` | Provide tool results back |
| `GenerationConfig` | `InteractionGenerationConfig` | Shared subset of fields |
| `Tool` | `InteractionTool` | Type-tagged polymorphic |

### Migration Checklist

1. **Start with simple text generation** — swap `generate_content()` for `create_interaction().execute()`
2. **Replace multi-turn logic** — use `with_previous_interaction()` instead of rebuilding history
3. **Update response parsing** — use `interaction.output_text()` instead of `response.candidates[0]...`
4. **Switch streaming** — handle `InteractionEvent` variants instead of raw chunks
5. **Map tools** — convert `Tool` to `InteractionTool` (function, google_search, code_execution, etc.)
6. **Explore new features** — try `with_background()`, `with_agent()`, or `with_environment()`

### Basic Content Generation

Get started with simple text generation, system prompts, and conversations. See [`basic_generation.rs`](examples/basic_generation.rs) for complete examples including simple messages, system prompts, and multi-turn conversations.

### Streaming Responses

Enable real-time content streaming for interactive applications. See [`basic_streaming.rs`](examples/basic_streaming.rs) for examples of processing content as it's generated with immediate display.

### Google Maps Grounding

Add location-aware capabilities to your applications with Google Maps integration. See [`simple_maps_example.rs`](examples/simple_maps_example.rs) for basic usage and [`google_maps_grounding.rs`](examples/google_maps_grounding.rs) for comprehensive examples.

### Token Count API

Calculate token usage before making generation requests for cost estimation and optimization. See [`count_tokens.rs`](examples/count_tokens.rs).

### Safety Settings

Customize content moderation with granular control over different harm categories and block thresholds. See [`safety_settings.rs`](examples/safety_settings.rs).

### Gemini 3 Pro

Access the latest model features including code execution and advanced thinking levels. See [`gemini_3_code_execution.rs`](examples/gemini_3_code_execution.rs) for code execution and [`gemini_3_thinking_and_media.rs`](examples/gemini_3_thinking_and_media.rs) for thinking levels.

## 🛠️ Key Features

The library provides comprehensive access to all Gemini 2.5 capabilities through an intuitive Rust API:

### 🧠 **Thinking Mode (Gemini 2.5)**

Advanced reasoning capabilities with thought process visibility and custom thinking budgets. See [`thinking_basic.rs`](examples/thinking_basic.rs) and [`thinking_advanced.rs`](examples/thinking_advanced.rs).

### 🛠️ **Function Calling & Tools**

- Custom function declarations with OpenAPI schema support (using `schemars`)
- Google Search integration for real-time information
- Google Maps grounding for location-aware responses
- Type-safe function definitions with automatic schema generation
- See [`tools.rs`](examples/tools.rs), [`complex_function.rs`](examples/complex_function.rs), and [`google_maps_grounding.rs`](examples/google_maps_grounding.rs)

### 🗺️ **Google Maps Grounding**

- **Location-Aware Responses**: Access Google Maps data for geographically specific queries
- **Widget Support**: Generate context tokens for interactive Google Maps widgets
- **Grounding Sources**: Access citation information for all Maps data used in responses
- **Easy Integration**: Simple API with location context configuration
- See [`simple_maps_example.rs`](examples/simple_maps_example.rs) and [`advanced_maps_configuration.rs`](examples/advanced_maps_configuration.rs)

### 🎨 **Multimodal Generation**

- **Image Generation**: Nano Banana (Flash) and Pro (Gemini 3) Text-to-image with detailed thinking and follow up editing capabilities
- **Speech Generation**: Text-to-speech with single and multi-speaker support
- **Image Processing**: Analyze images, videos, and binary data
- See [`image_generation.rs`](examples/image_generation.rs) and [`multi_speaker_tts.rs`](examples/multi_speaker_tts.rs)

### 📦 **Batch Processing**

Efficient processing of multiple requests with automatic file handling for large jobs. See [`batch_generate.rs`](examples/batch_generate.rs).

### 💾 **Content Caching**

Cache system instructions and conversation history to reduce costs and improve performance. See [`cache_basic.rs`](examples/cache_basic.rs).

### 📊 **Text Embeddings**

Advanced embedding generation with multiple task types for document retrieval and semantic search. See [`embedding.rs`](examples/embedding.rs).

### 🔍 **File Search (RAG)**

- **Semantic Document Search**: Upload documents and query them with natural language
- **Automatic Chunking**: Documents are automatically split, embedded, and indexed
- **Custom Metadata**: Filter searches using metadata tags (e.g., `category = "api-docs"`)
- **Grounding Citations**: Get source references for model responses
- **Multiple Upload Methods**: Direct upload or import from Files API
- **Persistent Storage**: Documents persist indefinitely until deleted
- See [`file_search_basic.rs`](examples/file_search_basic.rs), [`file_search_metadata.rs`](examples/file_search_metadata.rs), and [`file_search_import.rs`](examples/file_search_import.rs)

### 🔄 **Streaming Responses**

Real-time streaming of generated content for interactive applications. See [`streaming.rs`](examples/streaming.rs).

### ⚙️ **Highly Configurable**

- Custom models and endpoints
- Detailed generation parameters (temperature, tokens, etc.)
- HTTP client customization with timeouts and proxies
- See [`generation_config.rs`](examples/generation_config.rs) and [`custom_base_url.rs`](examples/custom_base_url.rs)

### 🔍 **Observability**

Built-in structured logging and telemetry with `tracing` for comprehensive monitoring and debugging.

### 🔢 **Token Count API**

Pre-calculate token usage for cost estimation and optimization. Calculate tokens for your requests before executing them. See [`count_tokens.rs`](examples/count_tokens.rs).

### 🎯 **Safety Settings**

Customize content moderation with granular control over different harm categories (Hate Speech, Dangerous Content, etc.) and block thresholds (Block None, Low, Medium, High). See [`safety_settings.rs`](examples/safety_settings.rs).

### 🚀 **Gemini 3 Pro**

- **Code Execution**: Generate and execute Python code for mathematical calculations, data analysis, and computational tasks
- **Thinking Levels**: Choose Low for faster responses or High for deeper analysis
- **Media Resolution**: Fine-grained control over image and PDF processing quality
- See [`gemini_3_code_execution.rs`](examples/gemini_3_code_execution.rs) and [`gemini_3_thinking_and_media.rs`](examples/gemini_3_thinking_and_media.rs)

### 📁 **File Handles**

Efficiently reference previously uploaded files without re-encoding. Upload files once and reference them multiple times, reducing data transfer. Supports PDFs, images, and other binary formats. See [`file_input.rs`](examples/file_input.rs) and [`files_usage.rs`](examples/files_usage.rs).

## 🔧 Configuration

### Custom Models

Configure different Gemini models including Flash, Pro, Lite, and custom models. See [`custom_models.rs`](examples/custom_models.rs) for examples of all model configuration options including convenience methods, enum variants, and custom model strings.

### Custom Base URL

Use custom API endpoints and configurations. See [`custom_base_url.rs`](examples/custom_base_url.rs) for examples of configuring custom endpoints with different models.

### Configurable HTTP Client Builder

For advanced HTTP configuration (timeouts, proxies, custom headers), use the builder pattern. See [`http_client_builder.rs`](examples/http_client_builder.rs) for a complete example with custom timeouts, user agents, connection pooling, and proxy configuration.

## 🔍 Tracing and Telemetry

The library is instrumented with the `tracing` crate to provide detailed telemetry data for monitoring and debugging. This allows you to gain deep insights into the library's performance and behavior.

Key tracing features include:

- **HTTP Request Tracing**: Captures detailed information about every API call, including HTTP method, URL, and response status, to help diagnose network-related issues
- **Token Usage Monitoring**: Records the number of prompt, candidate, and total tokens for each generation request, enabling cost analysis and optimization
- **Structured Logging**: Emits traces as structured events, compatible with modern log aggregation platforms like Elasticsearch, Datadog, and Honeycomb, allowing for powerful querying and visualization
- **Performance Metrics**: Provides timing information for each API request, allowing you to identify and address performance bottlenecks

To use these features, you will need to integrate a `tracing` subscriber into your application. See [`tracing_telemetry.rs`](examples/tracing_telemetry.rs) for comprehensive examples including basic console logging, structured logging for production, and environment-based log level filtering.

## 📚 Examples

The repository includes 30+ comprehensive examples demonstrating all features. See [`examples/README.md`](examples/README.md) for detailed information.

### Quick Start Examples

- [`basic_generation.rs`](examples/basic_generation.rs) - Simple content generation for beginners
- [`basic_streaming.rs`](examples/basic_streaming.rs) - Real-time streaming responses
- [`simple.rs`](examples/simple.rs) - Comprehensive example with function calling
- [`thinking_basic.rs`](examples/thinking_basic.rs) - Gemini 2.5 thinking mode
- [`count_tokens.rs`](examples/count_tokens.rs) - Pre-calculate token usage
- [`safety_settings.rs`](examples/safety_settings.rs) - Configure safety filters
- [`gemini_3_code_execution.rs`](examples/gemini_3_code_execution.rs) - Code execution with Python
- [`file_input.rs`](examples/file_input.rs) - Upload and reference files
- [`batch_generate.rs`](examples/batch_generate.rs) - Batch content generation
- [`image_generation.rs`](examples/image_generation.rs) - Text-to-image generation
- [`google_search.rs`](examples/google_search.rs) - Google Search integration
- [`url_context.rs`](examples/url_context.rs) - URL Context tool for web content analysis

### Interactions API Examples

- [`interaction_basic.rs`](examples/interaction_basic.rs) - Basic text generation (simplest starting point)
- [`interaction_multi_turn.rs`](examples/interaction_multi_turn.rs) - Multi-turn with `previous_interaction_id`
- [`interaction_streaming.rs`](examples/interaction_streaming.rs) - SSE step lifecycle events
- [`interaction_advanced.rs`](examples/interaction_advanced.rs) - Advanced configuration (tools, thinking, system prompt)
- [`interaction_function_calling.rs`](examples/interaction_function_calling.rs) - Function calling with `Step::FunctionCall`
- [`interaction_google_search.rs`](examples/interaction_google_search.rs) - Google Search grounding
- [`interaction_google_maps.rs`](examples/interaction_google_maps.rs) - Google Maps grounding
- [`interaction_code_execution.rs`](examples/interaction_code_execution.rs) - Python code execution
- [`interaction_thinking.rs`](examples/interaction_thinking.rs) - Thinking levels (low/medium/high)
- [`interaction_structured.rs`](examples/interaction_structured.rs) - JSON schema structured output
- [`interaction_multimodal.rs`](examples/interaction_multimodal.rs) - Image, audio, and video input
- [`interaction_image_gen.rs`](examples/interaction_image_gen.rs) - Image generation
- [`interaction_tts.rs`](examples/interaction_tts.rs) - Text-to-speech output
- [`interaction_background.rs`](examples/interaction_background.rs) - Background execution with polling
- [`interaction_deep_research.rs`](examples/interaction_deep_research.rs) - Deep Research managed agent
- [`interaction_antigravity.rs`](examples/interaction_antigravity.rs) - Antigravity agent with sandbox environment
- [`interaction_url_context.rs`](examples/interaction_url_context.rs) - URL context tool
- [`interaction_file_search.rs`](examples/interaction_file_search.rs) - RAG file search
- [`interaction_error_handling.rs`](examples/interaction_error_handling.rs) - Error handling patterns
- [`interaction_tracing.rs`](examples/interaction_tracing.rs) - Tracing and telemetry
- [`interaction_custom_client.rs`](examples/interaction_custom_client.rs) - Custom HTTP client configuration

Run any example:

```bash
GEMINI_API_KEY="your-api-key" cargo run --example basic_generation
```

## 🔑 API Key Setup

Get your API key from [Google AI Studio](https://aistudio.google.com/apikey) and set it as an environment variable:

```bash
export GEMINI_API_KEY="your-api-key-here"
```

## 🚦 Supported Models

### Standard Models (Both APIs)

- **Gemini 2.5 Flash** - Fast, efficient model (default) - `Model::Gemini25Flash`
- **Gemini 2.5 Flash Lite** - Lightweight model - `Model::Gemini25FlashLite`
- **Gemini 2.5 Pro** - Advanced model with thinking capabilities - `Model::Gemini25Pro`
- **Gemini 3 Pro** - Latest model with code execution and advanced thinking - `Model::Gemini3Pro` (Preview)
- **Gemini 3 Flash** - Fast model with thinking levels (Minimal, Low, Medium, High) - `Model::Gemini3Flash` (Preview)
- **Text Embedding 004** - Latest embedding model - `Model::TextEmbedding004`
- **Custom models** - Use `Model::Custom(String)` or string literals for other models

### Managed Agents (Interactions API only)

- **Deep Research** - Multi-step research agent with report generation - `AgentConfig::DeepResearch` or `.with_agent("deepresearch-2.5")`
- **Antigravity** - Code-writing agent with remote sandbox - `.with_agent("antigravity-preview-05-2026")`

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

For guidelines on developing agents and applications, see the [Agent Development Guide](AGENTS.md).

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Google for providing the Gemini API
- The Rust community for excellent async and HTTP libraries
- Special thanks to @npatsakula for major contributions that made this project more complete
- All contributors who have helped improve this library
