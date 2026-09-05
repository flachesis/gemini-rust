use serde::{Deserialize, Serialize};

// ============================================================================
// Content Types (polymorphic, type-tagged)
// ============================================================================

/// Input content for interactions — type-tagged polymorphic.
///
/// Unlike the generateContent API's `Part` enum, Interactions API content
/// uses a `type` discriminator field for each modality.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InteractionContent {
    Text {
        text: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        annotations: Vec<Annotation>,
    },
    Image {
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        uri: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<ImageMimeType>,
        #[serde(skip_serializing_if = "Option::is_none")]
        resolution: Option<MediaResolution>,
    },
    Audio {
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        uri: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<AudioMimeType>,
        #[serde(skip_serializing_if = "Option::is_none")]
        channels: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        sample_rate: Option<i32>,
    },
    Document {
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        uri: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<DocumentMimeType>,
    },
    Video {
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        uri: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<VideoMimeType>,
        #[serde(skip_serializing_if = "Option::is_none")]
        resolution: Option<MediaResolution>,
    },
}

impl InteractionContent {
    /// Create a text content item.
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text {
            text: text.into(),
            annotations: vec![],
        }
    }

    /// Create an image content item from base64 data.
    pub fn image(data: impl Into<String>, mime_type: ImageMimeType) -> Self {
        Self::Image {
            data: Some(data.into()),
            uri: None,
            mime_type: Some(mime_type),
            resolution: None,
        }
    }

    /// Create an image content item from a URI.
    pub fn image_uri(uri: impl Into<String>, mime_type: ImageMimeType) -> Self {
        Self::Image {
            data: None,
            uri: Some(uri.into()),
            mime_type: Some(mime_type),
            resolution: None,
        }
    }

    /// Create an audio content item from base64 data.
    pub fn audio(data: impl Into<String>, mime_type: AudioMimeType) -> Self {
        Self::Audio {
            data: Some(data.into()),
            uri: None,
            mime_type: Some(mime_type),
            channels: None,
            sample_rate: None,
        }
    }

    /// Create a video content item from a URI.
    pub fn video_uri(uri: impl Into<String>) -> Self {
        Self::Video {
            data: None,
            uri: Some(uri.into()),
            mime_type: None,
            resolution: None,
        }
    }

    /// Create a document content item from base64 data.
    pub fn document(data: impl Into<String>, mime_type: DocumentMimeType) -> Self {
        Self::Document {
            data: Some(data.into()),
            uri: None,
            mime_type: Some(mime_type),
        }
    }
}

// ============================================================================
// Annotation Types
// ============================================================================

/// Citation annotation — type-tagged polymorphic.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Annotation {
    UrlCitation {
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        start_index: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        end_index: Option<i64>,
    },
    FileCitation {
        #[serde(skip_serializing_if = "Option::is_none")]
        document_uri: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        file_name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        source: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        custom_metadata: Option<serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        page_number: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        media_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        start_index: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        end_index: Option<i64>,
    },
    PlaceCitation {
        #[serde(skip_serializing_if = "Option::is_none")]
        place_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        review_snippets: Option<ReviewSnippet>,
        #[serde(skip_serializing_if = "Option::is_none")]
        start_index: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        end_index: Option<i64>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReviewSnippet {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_id: Option<String>,
}

// ============================================================================
// Step Types (the core of Interactions API)
// ============================================================================

/// A single step in an interaction — type-tagged polymorphic.
///
/// Each step represents a typed action in the interaction history:
/// user input, model output, thoughts, function calls, tool invocations, etc.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Step {
    UserInput {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<InteractionContent>,
    },

    ModelOutput {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        content: Vec<InteractionContent>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<Status>,
    },

    Thought {
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        summary: Vec<ThoughtSummaryContent>,
    },

    FunctionCall {
        name: String,
        arguments: serde_json::Value,
        id: String,
    },

    FunctionResult {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        call_id: String,
        result: StepResult,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },

    CodeExecutionCall {
        arguments: CodeExecutionCallArguments,
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },

    CodeExecutionResult {
        result: String,
        call_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },

    UrlContextCall {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
        arguments: UrlContextCallArguments,
    },

    UrlContextResult {
        result: UrlContextResultData,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
        call_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },

    GoogleSearchCall {
        arguments: GoogleSearchCallArguments,
        #[serde(skip_serializing_if = "Option::is_none")]
        search_type: Option<GoogleSearchType>,
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },

    GoogleSearchResult {
        result: GoogleSearchResultItem,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
        call_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },

    GoogleMapsCall {
        #[serde(skip_serializing_if = "Option::is_none")]
        arguments: Option<GoogleMapsCallArguments>,
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },

    GoogleMapsResult {
        result: GoogleMapsResultItem,
        call_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },

    FileSearchCall {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },

    FileSearchResult {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        citations: Vec<Annotation>,
        call_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },

    McpServerToolCall {
        name: String,
        server_name: String,
        arguments: serde_json::Value,
        id: String,
    },

    McpServerToolResult {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        server_name: Option<String>,
        call_id: String,
        result: StepResult,
    },
}

// ============================================================================
// Step Sub-types
// ============================================================================

/// Function result can be a content array, JSON object, or string.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum StepResult {
    ContentArray(Vec<StepResultContent>),
    Object(serde_json::Value),
    String(String),
}

impl StepResult {
    pub fn from_string(s: impl Into<String>) -> Self {
        Self::String(s.into())
    }

    pub fn from_json(value: serde_json::Value) -> Self {
        Self::Object(value)
    }
}

/// Content items allowed in step results (text or image only).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StepResultContent {
    Text {
        text: String,
    },
    Image {
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        uri: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<ImageMimeType>,
    },
}

/// Thought summary content — polymorphic.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ThoughtSummaryContent {
    Text { text: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodeExecutionCallArguments {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<CodeLanguage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct UrlContextCallArguments {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub urls: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct GoogleSearchCallArguments {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub queries: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct GoogleMapsCallArguments {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub queries: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct GoogleSearchResultItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_suggestions: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct GoogleMapsResultItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub places: Option<GoogleMapsResultPlaces>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct GoogleMapsResultPlaces {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub place_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_snippets: Option<ReviewSnippet>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub widget_context_token: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct UrlContextResultData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<UrlContextStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Status {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub details: Vec<serde_json::Value>,
}

// ============================================================================
// Interaction Resource
// ============================================================================

/// Interaction resource — represents a complete interaction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Interaction {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(default)]
    pub status: InteractionStatus,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub steps: Vec<Step>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<InteractionUsage>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<InteractionTool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_interaction_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_modalities: Option<Vec<ResponseModality>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<ServiceTier>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_content: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_config: Option<AgentConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<InteractionError>,
}

/// Interaction status.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InteractionStatus {
    #[default]
    InProgress,
    RequiresAction,
    Completed,
    Failed,
    Cancelled,
    Incomplete,
    BudgetExceeded,
}

impl InteractionStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Completed
                | Self::Failed
                | Self::Cancelled
                | Self::Incomplete
                | Self::BudgetExceeded
        )
    }

    pub fn is_active(&self) -> bool {
        matches!(self, Self::InProgress | Self::RequiresAction)
    }
}

impl AsRef<str> for InteractionStatus {
    fn as_ref(&self) -> &str {
        match self {
            Self::InProgress => "in_progress",
            Self::RequiresAction => "requires_action",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Incomplete => "incomplete",
            Self::BudgetExceeded => "budget_exceeded",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InteractionError {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

// ============================================================================
// Tool Types
// ============================================================================

/// Tool declaration — type-tagged polymorphic.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InteractionTool {
    Function {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        parameters: Option<serde_json::Value>,
    },

    CodeExecution,

    UrlContext,

    ComputerUse {
        #[serde(skip_serializing_if = "Option::is_none")]
        environment: Option<ComputerUseEnvironment>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        excluded_predefined_functions: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        enable_prompt_injection_detection: Option<bool>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        disabled_safety_policies: Vec<ComputerUseSafetyPolicy>,
    },

    McpServer {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        headers: Option<serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        allowed_tools: Option<AllowedTools>,
    },

    GoogleSearch {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        search_types: Vec<GoogleSearchType>,
    },

    FileSearch {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        file_search_store_names: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        top_k: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata_filter: Option<String>,
    },

    GoogleMaps {
        #[serde(skip_serializing_if = "Option::is_none")]
        enable_widget: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        latitude: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        longitude: Option<f64>,
    },

    Retrieval {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        retrieval_types: Vec<RetrievalType>,
        #[serde(skip_serializing_if = "Option::is_none")]
        exa_ai_search_config: Option<ExaAISearchConfig>,
        #[serde(skip_serializing_if = "Option::is_none")]
        parallel_ai_search_config: Option<ParallelAISearchConfig>,
        #[serde(skip_serializing_if = "Option::is_none")]
        rag_store_config: Option<RagStoreConfig>,
    },
}

impl InteractionTool {
    /// Create a function tool.
    pub fn function(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: serde_json::Value,
    ) -> Self {
        Self::Function {
            name: Some(name.into()),
            description: Some(description.into()),
            parameters: Some(parameters),
        }
    }

    /// Create a Google Search tool.
    pub fn google_search() -> Self {
        Self::GoogleSearch {
            search_types: vec![GoogleSearchType::WebSearch],
        }
    }

    /// Create a code execution tool.
    pub fn code_execution() -> Self {
        Self::CodeExecution
    }

    /// Create a URL context tool.
    pub fn url_context() -> Self {
        Self::UrlContext
    }

    /// Create a Google Maps tool.
    pub fn google_maps() -> Self {
        Self::GoogleMaps {
            enable_widget: None,
            latitude: None,
            longitude: None,
        }
    }

    /// Create a file search tool.
    pub fn file_search(store_names: Vec<String>) -> Self {
        Self::FileSearch {
            file_search_store_names: store_names,
            top_k: None,
            metadata_filter: None,
        }
    }

    /// Create an MCP Server tool.
    pub fn mcp_server(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self::McpServer {
            name: Some(name.into()),
            url: Some(url.into()),
            headers: None,
            allowed_tools: None,
        }
    }

    /// Create a computer use tool.
    pub fn computer_use(environment: ComputerUseEnvironment) -> Self {
        Self::ComputerUse {
            environment: Some(environment),
            excluded_predefined_functions: vec![],
            enable_prompt_injection_detection: None,
            disabled_safety_policies: vec![],
        }
    }

    /// Create a retrieval tool.
    pub fn retrieval(retrieval_types: Vec<RetrievalType>) -> Self {
        Self::Retrieval {
            retrieval_types,
            exa_ai_search_config: None,
            parallel_ai_search_config: None,
            rag_store_config: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AllowedTools {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<ToolChoiceMode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ExaAISearchConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_config: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ParallelAISearchConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_config: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct RagStoreConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rag_resources: Option<RagResource>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct RagResource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rag_corpus: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rag_file_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rag_retrieval_config: Option<RagRetrievalConfig>,
}

/// Retrieval configuration for RAG queries.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct RagRetrievalConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hybrid_search: Option<HybridSearch>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<RagFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ranking: Option<Ranking>,
}

/// Hybrid search configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct HybridSearch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alpha: Option<f64>,
}

/// Filter configuration for RAG retrieval.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct RagFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector_distance_threshold: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector_similarity_threshold: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_filter: Option<String>,
}

/// Ranking and reranking configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Ranking {
    #[serde(flatten)]
    pub config: Option<serde_json::Value>,
}

// ============================================================================
// Response Format Types
// ============================================================================

/// Response format configuration — type-tagged polymorphic.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseFormat {
    Text {
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<TextMimeType>,
        #[serde(skip_serializing_if = "Option::is_none")]
        schema: Option<serde_json::Value>,
    },
    Audio {
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<AudioOutputMimeType>,
        #[serde(skip_serializing_if = "Option::is_none")]
        delivery: Option<DeliveryMode>,
        #[serde(skip_serializing_if = "Option::is_none")]
        sample_rate: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        bit_rate: Option<i32>,
    },
    Image {
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<ImageOutputMimeType>,
        #[serde(skip_serializing_if = "Option::is_none")]
        delivery: Option<DeliveryMode>,
        #[serde(skip_serializing_if = "Option::is_none")]
        aspect_ratio: Option<AspectRatio>,
        #[serde(skip_serializing_if = "Option::is_none")]
        image_size: Option<ImageSize>,
    },
    Video {
        #[serde(skip_serializing_if = "Option::is_none")]
        delivery: Option<DeliveryMode>,
        #[serde(skip_serializing_if = "Option::is_none")]
        gcs_uri: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        aspect_ratio: Option<VideoAspectRatio>,
        #[serde(skip_serializing_if = "Option::is_none")]
        duration: Option<String>,
    },
}

impl ResponseFormat {
    /// JSON structured output with a schema.
    pub fn json_schema(schema: serde_json::Value) -> Self {
        Self::Text {
            mime_type: Some(TextMimeType::ApplicationJson),
            schema: Some(schema),
        }
    }

    /// Plain text output.
    pub fn text() -> Self {
        Self::Text {
            mime_type: Some(TextMimeType::TextPlain),
            schema: None,
        }
    }
}

// ============================================================================
// Generation Config (Interactions version)
// ============================================================================

/// Model interaction configuration (mutually exclusive with agent_config).
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct InteractionGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stop_sequences: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_level: Option<InteractionThinkingLevel>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_summaries: Option<ThinkingSummaries>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<i64>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub speech_config: Vec<InteractionSpeechConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_config: Option<VideoConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoiceConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InteractionThinkingLevel {
    Minimal,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ThinkingSummaries {
    Auto,
    None,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct InteractionSpeechConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speaker: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct VideoConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<VideoTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolChoiceConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_tools: Option<AllowedTools>,
}

// ============================================================================
// Usage Types
// ============================================================================

/// Token usage statistics.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct InteractionUsage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_input_tokens: Option<i64>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub input_tokens_by_modality: Vec<ModalityTokens>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_cached_tokens: Option<i64>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cached_tokens_by_modality: Vec<ModalityTokens>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_output_tokens: Option<i64>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub output_tokens_by_modality: Vec<ModalityTokens>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tool_use_tokens: Option<i64>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_use_tokens_by_modality: Vec<ModalityTokens>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_thought_tokens: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<i64>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub grounding_tool_count: Vec<GroundingToolCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModalityTokens {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modality: Option<ResponseModality>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GroundingToolCount {
    #[serde(rename = "type")]
    pub grounding_type: Option<GroundingType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<i64>,
}

// ============================================================================
// Agent Config Types
// ============================================================================

/// Agent configuration (mutually exclusive with generation_config).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum AgentConfig {
    Dynamic,

    DeepResearch {
        #[serde(skip_serializing_if = "Option::is_none")]
        thinking_summaries: Option<ThinkingSummaries>,
        #[serde(skip_serializing_if = "Option::is_none")]
        visualization: Option<Visualization>,
        #[serde(skip_serializing_if = "Option::is_none")]
        collaborative_planning: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        enable_bigquery_tool: Option<bool>,
    },
}

// ============================================================================
// Enum Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ResponseModality {
    Text,
    Image,
    Audio,
    Video,
    Document,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ServiceTier {
    Flex,
    Standard,
    Priority,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryMode {
    Inline,
    Uri,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MediaResolution {
    Low,
    Medium,
    High,
    UltraHigh,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImageMimeType {
    #[serde(rename = "image/png")]
    Png,
    #[serde(rename = "image/jpeg")]
    Jpeg,
    #[serde(rename = "image/webp")]
    Webp,
    #[serde(rename = "image/heic")]
    Heic,
    #[serde(rename = "image/heif")]
    Heif,
    #[serde(rename = "image/gif")]
    Gif,
    #[serde(rename = "image/bmp")]
    Bmp,
    #[serde(rename = "image/tiff")]
    Tiff,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AudioMimeType {
    #[serde(rename = "audio/wav")]
    Wav,
    #[serde(rename = "audio/mp3")]
    Mp3,
    #[serde(rename = "audio/aac")]
    Aac,
    #[serde(rename = "audio/flac")]
    Flac,
    #[serde(rename = "audio/ogg")]
    Ogg,
    #[serde(rename = "audio/ogg_opus")]
    OggOpus,
    #[serde(rename = "audio/m4a")]
    M4a,
    #[serde(rename = "audio/opus")]
    Opus,
    #[serde(rename = "audio/pcm")]
    Pcm,
    #[serde(rename = "audio/l16")]
    L16,
    #[serde(rename = "audio/mulaw")]
    Mulaw,
    #[serde(rename = "audio/alaw")]
    Alaw,
    #[serde(rename = "audio/amr")]
    Amr,
    #[serde(rename = "audio/aiff")]
    Aiff,
    #[serde(rename = "audio/mpeg")]
    Mpeg,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VideoMimeType {
    #[serde(rename = "video/mp4")]
    Mp4,
    #[serde(rename = "video/webm")]
    Webm,
    #[serde(rename = "video/quicktime")]
    Quicktime,
    #[serde(rename = "video/mpeg")]
    Mpeg,
    #[serde(rename = "video/x-ms-wmv")]
    Wmv,
    #[serde(rename = "video/x-flv")]
    Flv,
    #[serde(rename = "video/3gpp")]
    ThreeGpp,
    #[serde(rename = "video/avi")]
    Avi,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DocumentMimeType {
    #[serde(rename = "application/pdf")]
    Pdf,
    #[serde(rename = "text/plain")]
    TextPlain,
    #[serde(rename = "text/html")]
    TextHtml,
    #[serde(rename = "text/css")]
    TextCss,
    #[serde(rename = "text/csv")]
    TextCsv,
    #[serde(rename = "text/javascript")]
    TextJavascript,
    #[serde(rename = "text/x-typescript")]
    TextXTypescript,
    #[serde(rename = "application/json")]
    ApplicationJson,
    #[serde(rename = "application/rtf")]
    Rtf,
    #[serde(rename = "application/vnd.openxmlformats-officedocument.wordprocessingml.document")]
    Docx,
    #[serde(rename = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")]
    Xlsx,
    #[serde(rename = "application/vnd.openxmlformats-officedocument.presentationml.presentation")]
    Pptx,
    #[serde(rename = "application/vnd.ms-excel")]
    Xls,
    #[serde(rename = "application/vnd.ms-powerpoint")]
    Ppt,
    #[serde(rename = "application/msword")]
    Doc,
    #[serde(rename = "text/markdown")]
    TextMarkdown,
    #[serde(rename = "text/x-python")]
    TextXPython,
    #[serde(rename = "application/x-java-source")]
    ApplicationXJavaSource,
    #[serde(rename = "application/x-sh")]
    ApplicationXSh,
    #[serde(rename = "application/typescript")]
    ApplicationTypescript,
    #[serde(rename = "application/x-typescript")]
    ApplicationXTypescript,
    #[serde(rename = "application/javascript")]
    ApplicationJavascript,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TextMimeType {
    #[serde(rename = "text/plain")]
    TextPlain,
    #[serde(rename = "application/json")]
    ApplicationJson,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AudioOutputMimeType {
    #[serde(rename = "audio/mp3")]
    Mp3,
    #[serde(rename = "audio/ogg_opus")]
    OggOpus,
    #[serde(rename = "audio/l16")]
    L16,
    #[serde(rename = "audio/wav")]
    Wav,
    #[serde(rename = "audio/alaw")]
    Alaw,
    #[serde(rename = "audio/mulaw")]
    Mulaw,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImageOutputMimeType {
    #[serde(rename = "image/jpeg")]
    Jpeg,
    #[serde(rename = "image/png")]
    Png,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AspectRatio {
    #[serde(rename = "1:1")]
    R1x1,
    #[serde(rename = "2:3")]
    R2x3,
    #[serde(rename = "3:2")]
    R3x2,
    #[serde(rename = "3:4")]
    R3x4,
    #[serde(rename = "4:3")]
    R4x3,
    #[serde(rename = "4:5")]
    R4x5,
    #[serde(rename = "5:4")]
    R5x4,
    #[serde(rename = "9:16")]
    R9x16,
    #[serde(rename = "16:9")]
    R16x9,
    #[serde(rename = "21:9")]
    R21x9,
    #[serde(rename = "1:8")]
    R1x8,
    #[serde(rename = "8:1")]
    R8x1,
    #[serde(rename = "1:4")]
    R1x4,
    #[serde(rename = "4:1")]
    R4x1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ImageSize {
    #[serde(rename = "512")]
    S512,
    #[serde(rename = "1K")]
    S1k,
    #[serde(rename = "2K")]
    S2k,
    #[serde(rename = "4K")]
    S4k,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VideoAspectRatio {
    #[serde(rename = "16:9")]
    R16x9,
    #[serde(rename = "9:16")]
    R9x16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum VideoTask {
    TextToVideo,
    ImageToVideo,
    ReferenceToVideo,
    Edit,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CodeLanguage {
    Python,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GoogleSearchType {
    WebSearch,
    ImageSearch,
    EnterpriseWebSearch,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalType {
    RagStore,
    ExaAiSearch,
    ParallelAiSearch,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ComputerUseEnvironment {
    Browser,
    Mobile,
    Desktop,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ComputerUseSafetyPolicy {
    DangerousContent,
    Harassment,
    HateSpeech,
    SexuallyExplicit,
    CivilIntegrity,
    Csam,
    TributeSpeech,
    Unspecified,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum UrlContextStatus {
    Success,
    Error,
    Paywall,
    Unsafe,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ToolChoiceMode {
    Auto,
    Any,
    None,
    Validated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Visualization {
    Off,
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GroundingType {
    GoogleSearch,
    GoogleMaps,
    Retrieval,
}

// ============================================================================
// Webhook Config
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct WebhookConfig {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub uris: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_metadata: Option<serde_json::Value>,
}

// ============================================================================
// Environment Config
// ============================================================================

/// Environment configuration — can be a full config object or just an ID string.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum EnvironmentConfigOrString {
    Config(EnvironmentConfig),
    Id(String),
}

/// Environment configuration — type-tagged polymorphic.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EnvironmentConfig {
    Remote {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        sources: Vec<EnvironmentSource>,
        #[serde(skip_serializing_if = "Option::is_none")]
        environment_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        network: Option<EnvironmentNetwork>,
    },
}

/// Environment source — type-tagged polymorphic.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EnvironmentSource {
    Gcs {
        #[serde(skip_serializing_if = "Option::is_none")]
        source: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        target: Option<String>,
    },
    Inline {
        #[serde(skip_serializing_if = "Option::is_none")]
        target: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        encoding: Option<String>,
    },
    Repository {
        #[serde(skip_serializing_if = "Option::is_none")]
        source: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        target: Option<String>,
    },
    SkillRegistry {
        #[serde(skip_serializing_if = "Option::is_none")]
        source: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        target: Option<String>,
    },
}

/// Network configuration — can be an allowlist object or "disabled" string.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum EnvironmentNetwork {
    Allowlist(EnvironmentNetworkEgressAllowlist),
    Disabled(String),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct EnvironmentNetworkEgressAllowlist {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowlist: Vec<AllowlistEntry>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AllowlistEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transform: Vec<serde_json::Value>,
}

// ============================================================================
// CreateInteractionRequest
// ============================================================================

/// Request body for creating an interaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CreateInteractionRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,

    pub input: InteractionInput,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<InteractionTool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<InteractionGenerationConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_config: Option<AgentConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_interaction_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<EnvironmentConfigOrString>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_content: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub response_modalities: Vec<ResponseModality>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<ServiceTier>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_config: Option<WebhookConfig>,
}

/// Input can be a string, single Content, Content array, or Step array.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InteractionInput {
    Text(String),
    Content(InteractionContent),
    ContentArray(Vec<InteractionContent>),
    StepArray(Vec<Step>),
}

impl InteractionInput {
    /// Create text input.
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text(text.into())
    }

    /// Create content array input.
    pub fn content_array(content: Vec<InteractionContent>) -> Self {
        Self::ContentArray(content)
    }

    /// Create step array input.
    pub fn step_array(steps: Vec<Step>) -> Self {
        Self::StepArray(steps)
    }
}

impl From<String> for InteractionInput {
    fn from(s: String) -> Self {
        Self::Text(s)
    }
}

impl From<&str> for InteractionInput {
    fn from(s: &str) -> Self {
        Self::Text(s.to_string())
    }
}

impl From<Vec<InteractionContent>> for InteractionInput {
    fn from(c: Vec<InteractionContent>) -> Self {
        Self::ContentArray(c)
    }
}

impl From<Vec<Step>> for InteractionInput {
    fn from(s: Vec<Step>) -> Self {
        Self::StepArray(s)
    }
}
