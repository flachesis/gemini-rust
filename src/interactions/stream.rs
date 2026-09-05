use std::pin::Pin;

use futures::Stream;
use serde::Deserialize;

use crate::client::Error;
use crate::interactions::model::*;

/// Interaction stream — yields `InteractionEvent` items.
pub type InteractionStream = Pin<Box<dyn Stream<Item = Result<InteractionEvent, Error>> + Send>>;

/// SSE event from the Interactions API stream.
///
/// Each event represents a lifecycle event in the interaction:
/// interaction creation/completion, step start/delta/stop, status updates, or errors.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "event_type")]
pub enum InteractionEvent {
    #[serde(rename = "interaction.created")]
    InteractionCreated {
        interaction: StreamInteraction,
        #[serde(default)]
        event_id: Option<String>,
        #[serde(default)]
        metadata: Option<StreamMetadata>,
    },

    #[serde(rename = "interaction.completed")]
    InteractionCompleted {
        interaction: StreamInteraction,
        #[serde(default)]
        event_id: Option<String>,
        #[serde(default)]
        metadata: Option<StreamMetadata>,
    },

    #[serde(rename = "interaction.status_update")]
    InteractionStatusUpdate {
        interaction_id: String,
        status: InteractionStatus,
        #[serde(default)]
        event_id: Option<String>,
        #[serde(default)]
        metadata: Option<StreamMetadata>,
    },

    #[serde(rename = "error")]
    Error {
        error: InteractionError,
        #[serde(default)]
        event_id: Option<String>,
        #[serde(default)]
        metadata: Option<StreamMetadata>,
    },

    #[serde(rename = "step.start")]
    StepStart {
        index: usize,
        step: Step,
        #[serde(default)]
        event_id: Option<String>,
        #[serde(default)]
        metadata: Option<StepDeltaMetadata>,
    },

    #[serde(rename = "step.delta")]
    StepDelta {
        index: usize,
        delta: StepDeltaData,
        #[serde(default)]
        event_id: Option<String>,
        #[serde(default)]
        metadata: Option<StepDeltaMetadata>,
    },

    #[serde(rename = "step.stop")]
    StepStop {
        index: usize,
        #[serde(default)]
        usage: Option<InteractionUsage>,
        #[serde(default)]
        step_usage: Option<InteractionUsage>,
        #[serde(default)]
        event_id: Option<String>,
        #[serde(default)]
        metadata: Option<StreamMetadata>,
    },
}

/// Partial Interaction resource in stream events (may omit some fields).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct StreamInteraction {
    pub id: Option<String>,
    pub object: Option<String>,
    pub model: Option<String>,
    pub agent: Option<String>,
    #[serde(default)]
    pub status: InteractionStatus,
    pub created: Option<String>,
    pub updated: Option<String>,
    pub service_tier: Option<ServiceTier>,
    pub usage: Option<InteractionUsage>,
    #[serde(default)]
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StreamMetadata {
    #[serde(default)]
    pub total_usage: Option<InteractionUsage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StepDeltaMetadata {
    #[serde(default)]
    pub total_usage: Option<InteractionUsage>,
}

/// Step delta data — type-tagged polymorphic.
///
/// Represents incremental updates to a step during streaming.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StepDeltaData {
    Text {
        text: String,
    },

    Image {
        #[serde(default)]
        data: Option<String>,
        #[serde(default)]
        uri: Option<String>,
        #[serde(default)]
        mime_type: Option<ImageMimeType>,
        #[serde(default)]
        resolution: Option<MediaResolution>,
    },

    Audio {
        #[serde(default)]
        data: Option<String>,
        #[serde(default)]
        uri: Option<String>,
        #[serde(default)]
        mime_type: Option<AudioMimeType>,
        #[serde(default)]
        sample_rate: Option<i32>,
        #[serde(default)]
        channels: Option<i32>,
    },

    Document {
        #[serde(default)]
        data: Option<String>,
        #[serde(default)]
        uri: Option<String>,
        #[serde(default)]
        mime_type: Option<DocumentMimeType>,
    },

    Video {
        #[serde(default)]
        data: Option<String>,
        #[serde(default)]
        uri: Option<String>,
        #[serde(default)]
        mime_type: Option<VideoMimeType>,
        #[serde(default)]
        resolution: Option<MediaResolution>,
    },

    ThoughtSummary {
        #[serde(default)]
        content: Option<InteractionContent>,
    },

    ThoughtSignature {
        #[serde(default)]
        signature: Option<String>,
    },

    TextAnnotationDelta {
        annotations: Vec<Annotation>,
    },

    ArgumentsDelta {
        #[serde(default)]
        arguments: Option<String>,
    },

    CodeExecutionCall {
        arguments: CodeExecutionCallArguments,
    },

    CodeExecutionResult {
        result: String,
        #[serde(default)]
        is_error: Option<bool>,
    },

    UrlContextCall {
        arguments: UrlContextCallArguments,
    },

    UrlContextResult {
        result: UrlContextResultData,
        #[serde(default)]
        is_error: Option<bool>,
    },

    GoogleSearchCall {
        arguments: GoogleSearchCallArguments,
    },

    GoogleSearchResult {
        result: GoogleSearchResultItem,
        #[serde(default)]
        is_error: Option<bool>,
    },

    GoogleMapsCall {
        #[serde(default)]
        arguments: Option<GoogleMapsCallArguments>,
    },

    GoogleMapsResult {
        #[serde(default)]
        result: Option<GoogleMapsResultItem>,
    },

    FileSearchCall,

    FileSearchResult {
        #[serde(default)]
        result: Option<FileSearchResultData>,
    },

    McpServerToolCall {
        name: String,
        server_name: String,
        arguments: serde_json::Value,
    },

    McpServerToolResult {
        #[serde(default)]
        name: Option<String>,
        #[serde(default)]
        server_name: Option<String>,
        result: StepResult,
    },

    FunctionResult {
        #[serde(default)]
        name: Option<String>,
        #[serde(default)]
        is_error: Option<bool>,
        call_id: String,
        result: StepResult,
    },
}

/// File search result data (for streaming deltas).
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
pub struct FileSearchResultData {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub citations: Vec<Annotation>,
}
