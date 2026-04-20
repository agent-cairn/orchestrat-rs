use thiserror::Error;

/// Result type for orchestrat-rs operations
pub type Result<T> = std::result::Result<T, OrchestratError>;

/// Errors that can occur during orchestration
#[derive(Error, Debug)]
pub enum OrchestratError {
    /// Error from Valkey operations
    #[error("Valkey error: {0}")]
    Valkey(#[from] redis::RedisError),

    /// Error from serialization/deserialization
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// LLM client error
    #[error("LLM client error: {0}")]
    LlmClient(String),

    /// Execution engine error
    #[error("Execution error: {0}")]
    Execution(String),

    /// Checkpoint error
    #[error("Checkpoint error: {0}")]
    Checkpoint(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),
}
