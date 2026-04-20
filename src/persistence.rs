use crate::checkpoint::Checkpoint;
use crate::error::{OrchestratError, Result};
use async_trait::async_trait;
use serde_json::json;

/// Configuration for Valkey persistence
#[derive(Debug, Clone)]
pub struct ValkeyPersistenceConfig {
    pub valkey_url: String,
    pub checkpoint_key_prefix: String,
    pub stream_name: String,
    pub ttl_seconds: u64,
}

impl Default for ValkeyPersistenceConfig {
    fn default() -> Self {
        Self {
            valkey_url: "redis://localhost:6379".to_string(),
            checkpoint_key_prefix: "orchestrat:checkpoint:".to_string(),
            stream_name: "orchestrat:events".to_string(),
            ttl_seconds: 86400, // 24 hours
        }
    }
}

/// Trait for persistence operations
#[async_trait]
pub trait Persistence: Send + Sync {
    /// Save a checkpoint to persistent storage
    async fn save_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()>;

    /// Load a checkpoint by execution ID
    async fn load_checkpoint(&self, execution_id: &str) -> Result<Option<Checkpoint>>;

    /// Delete a checkpoint
    async fn delete_checkpoint(&self, execution_id: &str) -> Result<()>;

    /// Publish an event to the event stream
    async fn publish_event(&self, event: serde_json::Value) -> Result<()>;
}

/// Valkey-based persistence implementation using redis crate (Valkey-compatible)
pub struct ValkeyPersistence {
    client: redis::aio::MultiplexedConnection,
    config: ValkeyPersistenceConfig,
}

impl ValkeyPersistence {
    /// Create a new ValkeyPersistence instance
    pub async fn new(config: ValkeyPersistenceConfig) -> Result<Self> {
        let client = redis::Client::open(config.valkey_url.as_str())
            .map_err(|e| crate::error::OrchestratError::Valkey(format!("Failed to create Redis client: {}", e)))?;
        let conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| crate::error::OrchestratError::Valkey(format!("Failed to connect to Valkey: {}", e)))?;

        Ok(Self { client: conn, config })
    }

    fn checkpoint_key(&self, execution_id: &str) -> String {
        format!("{}{}", self.config.checkpoint_key_prefix, execution_id)
    }
}

#[async_trait]
impl Persistence for ValkeyPersistence {
    async fn save_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()> {
        let key = self.checkpoint_key(&checkpoint.execution_id);
        let value = serde_json::to_string(checkpoint)
            .map_err(OrchestratError::from)?;

        // Save checkpoint with TTL
        let mut conn = self.client.clone();
        let _: () = redis::cmd("SETEX")
            .arg(&key)
            .arg(self.config.ttl_seconds)
            .arg(&value)
            .query_async(&mut conn)
            .await
            .map_err(|e| crate::error::OrchestratError::Valkey(format!("Failed to save checkpoint: {}", e)))?;

        // Publish checkpoint saved event
        self.publish_event(json!({
            "type": "checkpoint_saved",
            "execution_id": checkpoint.execution_id,
            "step_index": checkpoint.step_index,
            "timestamp": checkpoint.metadata.timestamp,
        }))
        .await?;

        Ok(())
    }

    async fn load_checkpoint(&self, execution_id: &str) -> Result<Option<Checkpoint>> {
        let key = self.checkpoint_key(execution_id);

        let mut conn = self.client.clone();
        let value: Option<String> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .map_err(|e| crate::error::OrchestratError::Valkey(format!("Failed to load checkpoint: {}", e)))?;

        match value {
            Some(v) => {
                let checkpoint: Checkpoint = serde_json::from_str(&v)
                    .map_err(|e| OrchestratError::from(e))?;
                Ok(Some(checkpoint))
            }
            None => Ok(None),
        }
    }

    async fn delete_checkpoint(&self, execution_id: &str) -> Result<()> {
        let key = self.checkpoint_key(execution_id);

        let mut conn = self.client.clone();
        let _: () = redis::cmd("DEL")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .map_err(|e| crate::error::OrchestratError::Valkey(format!("Failed to delete checkpoint: {}", e)))?;

        Ok(())
    }

    async fn publish_event(&self, event: serde_json::Value) -> Result<()> {
        let event_json = serde_json::to_string(&event)
            .map_err(|e| OrchestratError::from(e))?;

        // Use XADD to publish to Valkey Streams
        let mut conn = self.client.clone();
        let _: () = redis::cmd("XADD")
            .arg(&self.config.stream_name)
            .arg("*")
            .arg("event")
            .arg(&event_json)
            .query_async(&mut conn)
            .await
            .map_err(|e| crate::error::OrchestratError::Valkey(format!("Failed to publish event: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Valkey to be running
    async fn test_persistence_basic() {
        let config = ValkeyPersistenceConfig::default();
        let persistence = ValkeyPersistence::new(config).await.unwrap();

        let checkpoint = Checkpoint::new("test-persist", 0)
            .with_state("key", serde_json::json!("value"))
            .with_total_steps(5);

        // Save and load
        persistence.save_checkpoint(&checkpoint).await.unwrap();
        let loaded = persistence
            .load_checkpoint("test-persist")
            .await
            .unwrap()
            .unwrap();

        assert_eq!(loaded.execution_id, checkpoint.execution_id);
        assert_eq!(loaded.step_index, checkpoint.step_index);

        // Cleanup
        persistence.delete_checkpoint("test-persist").await.unwrap();
    }
}
