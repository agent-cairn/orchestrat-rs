use crate::checkpoint::Checkpoint;
use crate::error::{OrchestratError, Result};
use async_trait::async_trait;
use redis::AsyncCommands;
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
        let client = redis::Client::open(config.valkey_url.as_str())?;
        let conn = client.get_multiplexed_async_connection().await?;

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
        let value = serde_json::to_string(checkpoint)?;

        // Save checkpoint with TTL
        redis::cmd("SETEX")
            .arg(&key)
            .arg(self.config.ttl_seconds)
            .arg(&value)
            .query_async::<()>(&mut self.client.clone())
            .await?;

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

        let value: Option<String> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut self.client.clone())
            .await?;

        match value {
            Some(v) => {
                let checkpoint: Checkpoint = serde_json::from_str(&v)?;
                Ok(Some(checkpoint))
            }
            None => Ok(None),
        }
    }

    async fn delete_checkpoint(&self, execution_id: &str) -> Result<()> {
        let key = self.checkpoint_key(execution_id);
        redis::cmd("DEL")
            .arg(&key)
            .query_async::<()>(&mut self.client.clone())
            .await?;
        Ok(())
    }

    async fn publish_event(&self, event: serde_json::Value) -> Result<()> {
        let event_json = serde_json::to_string(&event)?;

        // Use XADD to publish to Valkey Streams
        let _: () = redis::cmd("XADD")
            .arg(&self.config.stream_name)
            .arg("*")
            .arg("event")
            .arg(event_json)
            .query_async(&mut self.client.clone())
            .await?;

        Ok(())
    }
}
