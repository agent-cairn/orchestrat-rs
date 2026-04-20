//! LLM client integration using rig-core
//!
//! Provides a wrapper around rig-core's LLM abstraction with retry logic
//! and streaming support.

use crate::error::{OrchestratError, Result};
use std::time::Duration;
use tokio::time::sleep;

/// LLM client configuration
#[derive(Debug, Clone)]
pub struct LlmConfig {
    /// Model identifier (e.g., "gpt-4", "claude-3-opus")
    pub model: String,

    /// API key for the LLM provider
    pub api_key: String,

    /// Maximum number of retries for failed requests
    pub max_retries: u32,

    /// Initial retry delay in milliseconds
    pub retry_delay_ms: u64,

    /// Maximum tokens for completion
    pub max_tokens: Option<usize>,

    /// Temperature for generation (0.0 - 2.0)
    pub temperature: Option<f32>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            model: "gpt-4".to_string(),
            api_key: String::new(),
            max_retries: 3,
            retry_delay_ms: 1000,
            max_tokens: None,
            temperature: None,
        }
    }
}

impl LlmConfig {
    pub fn new(model: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            api_key: api_key.into(),
            ..Default::default()
        }
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn with_retry_delay(mut self, delay_ms: u64) -> Self {
        self.retry_delay_ms = delay_ms;
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }
}

/// LLM client wrapper with retry logic
pub struct LlmClient {
    config: LlmConfig,
    // Note: rig-core client initialization will be added when rig-core API is finalized
    // For now, this is a placeholder structure
}

impl LlmClient {
    /// Create a new LLM client with the given configuration
    pub fn new(config: LlmConfig) -> Self {
        Self { config }
    }

    /// Send a completion request with automatic retry
    pub async fn complete(&self, prompt: &str) -> Result<String> {
        let mut last_error: Option<String> = None;

        for attempt in 0..=self.config.max_retries {
            match self.complete_once(prompt).await {
                Ok(response) => {
                    if attempt > 0 {
                        tracing::info!(
                            model = %self.config.model,
                            attempt,
                            "LLM request succeeded after retry"
                        );
                    }
                    return Ok(response);
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    last_error = Some(error_msg.clone());
                    tracing::warn!(
                        model = %self.config.model,
                        attempt,
                        error = %error_msg,
                        "LLM request failed"
                    );

                    if attempt < self.config.max_retries {
                        let delay = Duration::from_millis(
                            self.config.retry_delay_ms * 2_u64.pow(attempt as u32),
                        );
                        sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error
            .map(OrchestratError::LlmClient)
            .unwrap_or_else(|| OrchestratError::LlmClient("All retries exhausted".to_string())))
    }

    /// Single completion attempt without retry
    async fn complete_once(&self, _prompt: &str) -> Result<String> {
        // TODO: Implement actual rig-core integration when API is stable
        // This will use rig-core's ChatModel trait
        Err(OrchestratError::LlmClient(
            "rig-core integration pending API finalization".to_string(),
        ))
    }

    /// Stream a completion response
    pub async fn stream_complete<'a>(
        &'a self,
        _prompt: &'a str,
    ) -> Result<impl futures::Stream<Item = String> + 'a> {
        // TODO: Implement streaming with rig-core when API is stable
        use futures::stream;
        Ok(stream::empty())
    }

    /// Get the model identifier
    pub fn model(&self) -> &str {
        &self.config.model
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_config_default() {
        let config = LlmConfig::default();
        assert_eq!(config.model, "gpt-4");
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_llm_config_builder() {
        let config = LlmConfig::new("gpt-3.5-turbo", "test-key")
            .with_max_retries(5)
            .with_max_tokens(2048)
            .with_temperature(0.7);

        assert_eq!(config.model, "gpt-3.5-turbo");
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.max_tokens, Some(2048));
        assert_eq!(config.temperature, Some(0.7));
    }
}
