//! # gemini-rust
//!
//! A Rust client library for Google's Gemini 2.0 API.

mod batch;
mod batch_builder;
mod cache;
mod client;
mod content_builder;
mod embed_builder;
mod files;
mod models;
mod tools;

#[cfg(test)]
mod tests;

pub use batch::{Batch, BatchGenerationResponseItem, BatchStatus, Error as BatchError};
pub use batch_builder::BatchBuilder;
pub use cache::{CacheBuilder, CachedContentHandle, Error as CacheError};
pub use client::{Error as ClientError, Gemini, Model};
pub use content_builder::ContentBuilder;
pub use files::{FileBuilder, GeminiFile};
pub use models::{
    BatchConfig, BatchGenerateContentResponseItem, BatchMetadata, BatchOperationResponse,
    BatchRequestFileItem, BatchRequestItem, BatchResponseFileItem, BatchState, BatchStats, Blob,
    BlockReason, CacheExpirationRequest, CacheExpirationResponse, CacheUsageMetadata,
    CachedContent, CachedContentSummary, Candidate, CitationMetadata, Content,
    CreateCachedContentRequest, DeleteCachedContentResponse, File, FileState, FinishReason,
    FunctionCallingConfig, FunctionCallingMode, GenerateContentRequest, GenerationConfig,
    GenerationResponse, HarmBlockThreshold, HarmCategory, HarmProbability, InlinedResponses,
    InputConfig, ListCachedContentsResponse, Message, Modality, MultiSpeakerVoiceConfig,
    OperationResult, Part, PrebuiltVoiceConfig, PromptTokenDetails, RequestMetadata,
    RequestsContainer, Role, SafetyRating, SpeakerVoiceConfig, SpeechConfig, TaskType,
    ThinkingConfig, ToolConfig, UsageMetadata, VoiceConfig,
};

pub use tools::{
    FunctionCall, FunctionDeclaration, FunctionParameters, FunctionResponse, PropertyDetails, Tool,
};
