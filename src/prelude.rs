//! # Prelude for the Gemini Rust Crate
//!
//! This module re-exports the most commonly used types for convenience.
//!
//! Instead of importing each type individually, you can simply use:
//!
//! ```rust,ignore
//! use gemini_rust::prelude::*;
//! ```

#[cfg(feature = "batch")]
pub use crate::batch::{Batch, BatchBuilder, BatchError, BatchStatus};
#[cfg(feature = "cache")]
pub use crate::cache::{CacheBuilder, CacheError, CachedContentHandle};
#[cfg(feature = "embedding")]
pub use crate::embedding::{EmbedBuilder, TaskType};
#[cfg(feature = "files")]
pub use crate::files::{FileBuilder, GeminiFile};
#[cfg(feature = "generation")]
pub use crate::generation::{ContentBuilder, GenerationConfig, GenerationResponse};

pub use crate::client::{Error as ClientError, Gemini, Model};
pub use crate::models::{Content, Message, Part, Role};
pub use crate::safety::{HarmBlockThreshold, HarmCategory, SafetySetting};
pub use crate::tools::{
    FunctionCall, FunctionDeclaration, FunctionParameters, FunctionResponse, PropertyDetails, Tool,
};
