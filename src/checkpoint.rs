use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Checkpoint data structure for saving and restoring execution state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Unique identifier for this execution run
    pub execution_id: String,

    /// Current step index in the execution sequence
    pub step_index: usize,

    /// Execution state (arbitrary key-value data)
    pub state: HashMap<String, serde_json::Value>,

    /// Metadata about the execution
    pub metadata: CheckpointMetadata,
}

/// Metadata associated with a checkpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointMetadata {
    /// Timestamp when checkpoint was created
    pub timestamp: u64,

    /// Model being used
    pub model: String,

    /// Total number of steps in the execution plan
    pub total_steps: Option<usize>,

    /// Additional custom metadata
    pub extra: HashMap<String, String>,
}

impl Checkpoint {
    pub fn new(execution_id: impl Into<String>, step_index: usize) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            execution_id: execution_id.into(),
            step_index,
            state: HashMap::new(),
            metadata: CheckpointMetadata {
                timestamp,
                model: "default".to_string(),
                total_steps: None,
                extra: HashMap::new(),
            },
        }
    }

    pub fn with_state(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.state.insert(key.into(), value);
        self
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.metadata.model = model.into();
        self
    }

    pub fn with_total_steps(mut self, total: usize) -> Self {
        self.metadata.total_steps = Some(total);
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.extra.insert(key.into(), value.into());
        self
    }

    pub fn get_state(&self, key: &str) -> Option<&serde_json::Value> {
        self.state.get(key)
    }

    pub fn is_complete(&self) -> bool {
        if let Some(total) = self.metadata.total_steps {
            self.step_index >= total
        } else {
            false
        }
    }
}
