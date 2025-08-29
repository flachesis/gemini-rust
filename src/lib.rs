//! # gemini-rust
//!
//! A Rust client library for Google's Gemini 2.0 API.

mod batch_builder;
mod client;
mod content_builder;
mod embed_builder;
mod error;
mod models;
mod tools;

pub use batch_builder::BatchBuilder;
pub use client::Gemini;
pub use content_builder::ContentBuilder;
pub use error::Error;
pub use models::{
    BatchConfig, BatchMetadata, BatchRequestItem, BatchStats, Blob, Candidate, CitationMetadata,
    Content, FunctionCallingMode, GenerateContentRequest, GenerationConfig, GenerationResponse,
    InputConfig, Message, Part, PromptTokenDetails, RequestMetadata, RequestsContainer, Role,
    SafetyRating, TaskType, ThinkingConfig, UsageMetadata,
};

pub use tools::{FunctionCall, FunctionDeclaration, FunctionParameters, PropertyDetails, Tool};

/// Result type for this crate
pub type Result<T> = std::result::Result<T, Error>;
