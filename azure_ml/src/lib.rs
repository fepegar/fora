//! Azure ML REST API client
//!
//! This crate provides a high-level client for interacting with Azure ML services.

pub mod client;
pub mod models;

// Re-export the main client
pub use client::{AzureMLClient, AzureMLConfig};

// Re-export all models for convenience
pub use models::*;

/// Result type for Azure ML operations
pub type Result<T> = std::result::Result<T, AzureMLError>;

/// Error types for Azure ML operations
#[derive(Debug, thiserror::Error)]
pub enum AzureMLError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Azure authentication error: {0}")]
    Authentication(#[from] azure_core::Error),

    #[error("Azure ML API error: {status_code} - {message}")]
    Api { status_code: u16, message: String },

    #[error("Invalid configuration: {0}")]
    Configuration(String),

    #[error("Resource not found: {0}")]
    NotFound(String),
}
