//! # gemini-rust
//!
//! A Rust client library for Google's Gemini 2.0 API.

mod batch;
mod batch_builder;
mod client;
mod content_builder;
mod embed_builder;
mod files;
mod models;
mod tools;

#[cfg(test)]
mod tests;

pub use batch::{Batch, BatchStatus, Error as BatchError};
pub use batch_builder::BatchBuilder;
pub use client::{Error as ClientError, Gemini, Model};
pub use content_builder::ContentBuilder;
pub use files::{FileBuilder, GeminiFile};
pub use models::{
    BatchConfig, BatchFileItem, BatchGenerateContentResponseItem, BatchMetadata,
    BatchOperationResponse, BatchRequestItem, BatchResultItem, BatchState, BatchStats, Blob,
    BlockReason, Candidate, CitationMetadata, Content, File, FileState, FinishReason,
    FunctionCallingConfig, FunctionCallingMode, GenerateContentRequest, GenerationConfig,
    GenerationResponse, HarmBlockThreshold, HarmCategory, HarmProbability, InlinedResponses,
    InputConfig, Message, Modality, MultiSpeakerVoiceConfig, OperationResult, Part,
    PrebuiltVoiceConfig, PromptTokenDetails, RequestMetadata, RequestsContainer, Role,
    SafetyRating, SpeakerVoiceConfig, SpeechConfig, TaskType, ThinkingConfig, ToolConfig,
    UsageMetadata, VoiceConfig,
};

pub use tools::{
    FunctionCall, FunctionDeclaration, FunctionParameters, FunctionResponse, PropertyDetails, Tool,
};
