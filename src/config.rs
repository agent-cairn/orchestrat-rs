use serde::{Deserialize, Serialize};

/// Configuration for the orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    /// Valkey connection URL
    pub valkey_url: String,

    /// LLM model identifier
    pub model: String,

    /// Maximum retries for failed operations
    pub max_retries: u32,

    /// Interval for automatic checkpointing (in steps)
    pub checkpoint_interval: usize,

    /// Whether to enable persistence
    pub persistence_enabled: bool,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            valkey_url: "redis://localhost:6379".to_string(),
            model: "gpt-4".to_string(),
            max_retries: 3,
            checkpoint_interval: 10,
            persistence_enabled: true,
        }
    }
}

impl OrchestratorConfig {
    pub fn new(valkey_url: impl Into<String>) -> Self {
        Self {
            valkey_url: valkey_url.into(),
            ..Default::default()
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn with_checkpoint_interval(mut self, interval: usize) -> Self {
        self.checkpoint_interval = interval;
        self
    }

    pub fn with_persistence(mut self, enabled: bool) -> Self {
        self.persistence_enabled = enabled;
        self
    }
}
