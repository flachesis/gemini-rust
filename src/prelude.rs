//! # Gemini Rust SDK Prelude
//!
//! This module provides convenient imports for the most commonly used types
//! in the Gemini Rust SDK. Import everything with:
//!
//! ```rust
//! use gemini_rust::prelude::*;
//! ```
//!
//! This prelude includes only the essential types that most users will need.
//! For more specialized types, import them directly from the crate root or
//! their respective modules.

// Core client types
pub use crate::{ClientError, Gemini, Model};

// Builders for creating requests
#[allow(deprecated)]
pub use crate::{ContentBuilder, EmbedBuilder};

// Core data types for messages and content
pub use crate::{Content, FileData, Message, Role};

// Main response types
#[allow(deprecated)]
pub use crate::{ContentEmbeddingResponse, CountTokensResponse, GenerationResponse};

// Configuration types
#[allow(deprecated)]
pub use crate::{GenerationConfig, TaskType};

// Safety settings
pub use crate::{HarmBlockThreshold, HarmCategory, SafetySetting};

// Function calling
pub use crate::{FunctionDeclaration, FunctionResponse, Tool};

// Batch and file handles (commonly used for async operations)
pub use crate::{Batch, FileHandle};

// File Search for RAG
pub use crate::{
    ChunkingConfig, CustomMetadata, CustomMetadataValue, DocumentHandle, FileSearchStoreHandle,
    OperationHandle, WhiteSpaceConfig,
};

// Interactions API — the modern way to use Gemini
pub use crate::{
    InteractionBuilder, InteractionHandle, Interaction, InteractionStatus, InteractionInput,
    InteractionContent, InteractionTool, Step, ResponseFormat, InteractionGenerationConfig,
    InteractionUsage, InteractionThinkingLevel, AgentConfig, InteractionStream,
    InteractionEvent, StepDeltaData, ThinkingSummaries, ThoughtSummaryContent, StepResult,
    Annotation, ImageMimeType, AudioMimeType, VideoMimeType, DocumentMimeType,
    AudioOutputMimeType, ImageOutputMimeType, TextMimeType, DeliveryMode, AspectRatio,
    ImageSize, VideoAspectRatio, MediaResolution, ResponseModality, ServiceTier,
    EnvironmentConfig, EnvironmentConfigOrString, EnvironmentSource, WebhookConfig,
    CodeLanguage, GoogleSearchType, CodeExecutionCallArguments,
    InteractionSpeechConfig, VideoConfig, ToolChoiceConfig, AllowedTools, Visualization,
};
