//! # Gemini API Data Models for Text Embeddings
//!
//! This module contains data structures for generating text embeddings with the
//! Gemini API. Embeddings are vector representations of text that can be used for
//! tasks like semantic similarity, classification, and clustering.

use crate::models::Content;
use serde::{Deserialize, Serialize};

/// Content of the embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentEmbedding {
    /// The values generated
    pub values: Vec<f32>, //Maybe Quantize this
}

/// Response from the Gemini API for content embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentEmbeddingResponse {
    /// The embeddings generated
    pub embedding: ContentEmbedding,
}

/// Response from the Gemini API for batch content embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchContentEmbeddingResponse {
    /// The embeddings generated
    pub embeddings: Vec<ContentEmbedding>,
}

/// Request to embed words
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedContentRequest {
    /// The specified embedding model
    pub model: String,
    /// The chunks content to generate embeddings
    pub content: Content,
    /// The embedding task type (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_type: Option<TaskType>,
    /// The title of the document (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// The output_dimensionality (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_dimensionality: Option<i32>,
}

/// Request to batch embed requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchEmbedContentsRequest {
    /// The list of embed requests
    pub requests: Vec<EmbedContentRequest>,
}

/// Embedding Task types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskType {
    /// Used to generate embeddings that are optimized to assess text similarity
    SemanticSimilarity,
    /// Used to generate embeddings that are optimized to classify texts according to preset labels
    Classification,
    /// Used to generate embeddings that are optimized to cluster texts based on their similarities
    Clustering,

    /// Used to generate embeddings that are optimized for document search or information retrieval
    RetrievalDocument,
    /// Used to generate embeddings optimized for search queries
    RetrievalQuery,
    /// Used to generate embeddings optimized for question answering tasks
    QuestionAnswering,
    /// Used to generate embeddings optimized for fact verification
    FactVerification,

    /// Used to retrieve a code block based on a natural language query, such as sort an array or reverse a linked list.
    /// Embeddings of the code blocks are computed using RETRIEVAL_DOCUMENT.
    CodeRetrievalQuery,
}
