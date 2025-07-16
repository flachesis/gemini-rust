//! # gemini-rust
//!
//! A Rust client library for Google's Gemini 2.0 API.

mod client;
mod content_builder;
mod embed_builder;
mod error;
mod models;
mod tools;

pub use client::Gemini;
pub use error::Error;
pub use models::{
    Blob, Candidate, CitationMetadata, Content, FunctionCallingMode, GenerationConfig,
    GenerationResponse, Message, Part, Role, SafetyRating, TaskType, ThinkingConfig, UsageMetadata,
};

pub use tools::{FunctionCall, FunctionDeclaration, FunctionParameters, PropertyDetails, Tool};

/// Result type for this crate
pub type Result<T> = std::result::Result<T, Error>;
