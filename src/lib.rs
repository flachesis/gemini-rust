//! # gemini-rust
//!
//! A Rust client library for Google's Gemini 2.0 API.

// Declare modules with feature flags
#[cfg(feature = "batch")]
pub mod batch;
#[cfg(feature = "cache")]
pub mod cache;
#[cfg(feature = "embedding")]
pub mod embedding;
#[cfg(feature = "files")]
pub mod files;
#[cfg(feature = "generation")]
pub mod generation;

pub mod safety;
pub mod tools;

pub mod client;
pub mod common;
pub mod models;
pub mod prelude;

#[cfg(test)]
mod tests;

pub use client::{Error as ClientError, Gemini, Model};
pub use models::{Blob, Content, Message, Part, Role};
