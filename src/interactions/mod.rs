//! Interactions API — the modern interface for Gemini models and agents.
//!
//! The Interactions API is the simplest and best way to use Gemini models and agents.
//! It provides a unified interface for all use cases, including single-turn text generation,
//! multimodal understanding, structured output, tool orchestration, and agentic workflows.
//!
//! # Key Advantages
//!
//! - Unified interface for models and agents
//! - Server-side state management (`previous_interaction_id`)
//! - Observable execution steps
//! - Background execution
//! - Higher cache hit rates
//!
//! # Quick Start
//!
//! ```no_run
//! # use gemini_rust::prelude::*;
//! # async fn example(gemini: &Gemini) -> Result<(), Box<dyn std::error::Error>> {
//! let interaction = gemini.create_interaction()
//!     .with_model("gemini-2.5-flash")
//!     .with_text("Hello, world!")
//!     .execute()
//!     .await?;
//!
//! println!("{}", interaction.output_text());
//! # Ok(())
//! # }
//! ```

pub mod builder;
pub mod handle;
pub mod model;
pub mod stream;

pub use builder::InteractionBuilder;
pub use handle::InteractionHandle;
pub use model::*;
pub use stream::{InteractionEvent, InteractionStream, StepDeltaData};

/// Convenience methods on [`Interaction`].
impl Interaction {
    /// Get the concatenated final text output.
    ///
    /// Extracts text from the last `model_output` step's text content items.
    pub fn output_text(&self) -> String {
        self.steps
            .iter()
            .rev()
            .find(|s| matches!(s, Step::ModelOutput { .. }))
            .and_then(|s| {
                if let Step::ModelOutput { content, .. } = s {
                    Some(
                        content
                            .iter()
                            .filter_map(|c| {
                                if let InteractionContent::Text { text, .. } = c {
                                    Some(text.clone())
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(""),
                    )
                } else {
                    None
                }
            })
            .unwrap_or_default()
    }

    /// Get all function_call steps.
    pub fn function_calls(&self) -> Vec<&Step> {
        self.steps
            .iter()
            .filter(|s| matches!(s, Step::FunctionCall { .. }))
            .collect()
    }

    /// Get all model_output steps.
    pub fn model_outputs(&self) -> Vec<&Step> {
        self.steps
            .iter()
            .filter(|s| matches!(s, Step::ModelOutput { .. }))
            .collect()
    }

    /// Get all thought steps.
    pub fn thoughts(&self) -> Vec<&Step> {
        self.steps
            .iter()
            .filter(|s| matches!(s, Step::Thought { .. }))
            .collect()
    }

    /// Whether the interaction requires user action (e.g., function calling).
    pub fn requires_action(&self) -> bool {
        self.status == InteractionStatus::RequiresAction
    }

    /// Whether the interaction is completed.
    pub fn is_completed(&self) -> bool {
        self.status == InteractionStatus::Completed
    }

    /// Get the output image (last model-generated image).
    pub fn output_image(&self) -> Option<&InteractionContent> {
        self.steps.iter().rev().find_map(|s| {
            if let Step::ModelOutput { content, .. } = s {
                content
                    .iter()
                    .find(|c| matches!(c, InteractionContent::Image { .. }))
            } else {
                None
            }
        })
    }

    /// Get the output audio (last model-generated audio).
    pub fn output_audio(&self) -> Option<&InteractionContent> {
        self.steps.iter().rev().find_map(|s| {
            if let Step::ModelOutput { content, .. } = s {
                content
                    .iter()
                    .find(|c| matches!(c, InteractionContent::Audio { .. }))
            } else {
                None
            }
        })
    }

    /// Get the output video (last model-generated video).
    pub fn output_video(&self) -> Option<&InteractionContent> {
        self.steps.iter().rev().find_map(|s| {
            if let Step::ModelOutput { content, .. } = s {
                content
                    .iter()
                    .find(|c| matches!(c, InteractionContent::Video { .. }))
            } else {
                None
            }
        })
    }

    /// Get all citation annotations from model outputs.
    pub fn citations(&self) -> Vec<&Annotation> {
        self.steps
            .iter()
            .filter_map(|s| {
                if let Step::ModelOutput { content, .. } = s {
                    Some(content.iter().flat_map(|c| {
                        if let InteractionContent::Text { annotations, .. } = c {
                            annotations.iter().collect::<Vec<_>>()
                        } else {
                            vec![]
                        }
                    }))
                } else {
                    None
                }
            })
            .flatten()
            .collect()
    }

    /// Get the total token count.
    pub fn total_tokens(&self) -> Option<i64> {
        self.usage.as_ref()?.total_tokens
    }

    /// Get the interaction ID.
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }
}
