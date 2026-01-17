# gemini-rust

A comprehensive Rust client library for Google's Gemini API.

[![Crates.io](https://img.shields.io/crates/v/gemini-rust.svg)](https://crates.io/crates/gemini-rust)
[![Documentation](https://docs.rs/gemini-rust/badge.svg)](https://docs.rs/gemini-rust)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## ✨ Features

- **🚀 Complete Gemini API Implementation** - Full support for all Gemini API endpoints
- **🛠️ Function Calling & Tools** - Custom functions, Google Search, and Google Maps integration with OpenAPI schema support
- **🗺️ Google Maps Grounding** - Location-aware responses with Google Maps data and widget support
- **📦 Batch Processing** - Efficient batch content generation and embedding
- **💾 Content Caching** - Cache system instructions and conversation history for cost optimization
- **🔄 Streaming Responses** - Real-time streaming of generated content
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
gemini-rust = "1.6.1"
```

## 🚀 Quick Start

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

- **Gemini 2.5 Flash** - Fast, efficient model (default) - `Model::Gemini25Flash`
- **Gemini 2.5 Flash Lite** - Lightweight model - `Model::Gemini25FlashLite`
- **Gemini 2.5 Pro** - Advanced model with thinking capabilities - `Model::Gemini25Pro`
- **Gemini 3 Pro** - Latest model with code execution and advanced thinking - `Model::Gemini3Pro` (Preview)
- **Text Embedding 004** - Latest embedding model - `Model::TextEmbedding004`
- **Custom models** - Use `Model::Custom(String)` or string literals for other models

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
