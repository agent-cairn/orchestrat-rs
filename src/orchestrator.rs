use crate::checkpoint::Checkpoint;
use crate::config::OrchestratorConfig;
use crate::error::Result;
use crate::executor::{ContinuationStep, ExecutionEngine, ExecutionPlan};
use crate::persistence::{Persistence, ValkeyPersistence, ValkeyPersistenceConfig};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Main orchestrator for LLM workflows
pub struct Orchestrator {
    config: OrchestratorConfig,
    execution_engine: ExecutionEngine,
    persistence: Arc<dyn Persistence>,
    // In-memory cache of active checkpoints
    active_checkpoints: Arc<RwLock<std::collections::HashMap<String, Checkpoint>>>,
}

impl Orchestrator {
    /// Create a new orchestrator with the given configuration
    pub async fn new(config: OrchestratorConfig) -> Result<Self> {
        // Initialize persistence layer
        let persistence_config = ValkeyPersistenceConfig {
            valkey_url: config.valkey_url.clone(),
            checkpoint_key_prefix: "orchestrat:checkpoint:".to_string(),
            stream_name: "orchestrat:events".to_string(),
            ttl_seconds: 86400,
        };

        let persistence = Arc::new(ValkeyPersistence::new(persistence_config).await?);

        // Create empty execution plan
        let plan = ExecutionPlan::new("default-plan");
        let execution_engine = ExecutionEngine::new(plan).with_checkpoint_interval(config.checkpoint_interval);

        Ok(Self {
            config,
            execution_engine,
            persistence,
            active_checkpoints: Arc::new(RwLock::new(std::collections::HashMap::new())),
        })
    }

    /// Register a new step in the workflow
    pub fn register_step(&mut self, _step: ContinuationStep) {
        // Note: This would require rebuilding the engine with a new plan
        // For now, this is a placeholder for the API
        tracing::warn!("register_step requires rebuilding the execution engine - not implemented yet");
    }

    /// Start a new execution workflow
    pub async fn start_execution(&self, execution_id: Option<String>) -> Result<String> {
        let execution_id = execution_id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let total_steps = self.execution_engine.step_count();

        let checkpoint = Checkpoint::new(&execution_id, 0)
            .with_model(&self.config.model)
            .with_total_steps(total_steps);

        let timestamp = checkpoint.metadata.timestamp;

        // Cache in memory
        self.active_checkpoints
            .write()
            .await
            .insert(execution_id.clone(), checkpoint);

        // Publish start event
        self.persistence
            .publish_event(json!({
                "type": "execution_started",
                "execution_id": execution_id,
                "model": self.config.model,
                "total_steps": total_steps,
                "timestamp": timestamp,
            }))
            .await?;

        // Save initial checkpoint
        if self.config.persistence_enabled {
            let checkpoint = self.active_checkpoints.read().await.get(&execution_id).cloned();
            if let Some(cp) = checkpoint {
                self.persistence.save_checkpoint(&cp).await?;
            }
        }

        tracing::info!(execution_id = %execution_id, "Started new execution");

        Ok(execution_id)
    }

    /// Resume an existing execution from its checkpoint
    pub async fn resume_execution(&self, execution_id: &str) -> Result<Checkpoint> {
        // Try to load from persistence
        if self.config.persistence_enabled {
            if let Some(checkpoint) = self.persistence.load_checkpoint(execution_id).await? {
                tracing::info!(
                    execution_id = %execution_id,
                    step_index = checkpoint.step_index,
                    "Resumed execution from checkpoint"
                );
                return Ok(checkpoint);
            }
        }

        // Check in-memory cache
        if let Some(checkpoint) = self.active_checkpoints.read().await.get(execution_id) {
            tracing::info!(
                execution_id = %execution_id,
                step_index = checkpoint.step_index,
                "Resumed execution from memory cache"
            );
            return Ok(checkpoint.clone());
        }

        Err(crate::error::OrchestratError::Execution(format!(
            "No checkpoint found for execution_id: {}",
            execution_id
        )))
    }

    /// Run the workflow to completion
    pub async fn run(&self, execution_id: &str) -> Result<Checkpoint> {
        let mut checkpoint = self.resume_execution(execution_id).await?;

        // Determine if checkpointing is needed
        let checkpoint_interval = self.config.checkpoint_interval;
        let persistence_enabled = self.config.persistence_enabled;

        checkpoint = self
            .execution_engine
            .execute_from(checkpoint, move |cp| {
                // Checkpoint at interval or on error
                cp.step_index % checkpoint_interval == 0
            })
            .await?;

        // Update in-memory cache
        self.active_checkpoints
            .write()
            .await
            .insert(execution_id.to_string(), checkpoint.clone());

        // Save final checkpoint
        if persistence_enabled {
            self.persistence.save_checkpoint(&checkpoint).await?;
        }

        // Publish completion event
        self.persistence
            .publish_event(json!({
                "type": "execution_completed",
                "execution_id": execution_id,
                "success": true,
                "final_step": checkpoint.step_index,
                "timestamp": checkpoint.metadata.timestamp,
            }))
            .await?;

        Ok(checkpoint)
    }

    /// Get the current state of an execution
    pub async fn get_state(&self, execution_id: &str) -> Result<Checkpoint> {
        self.resume_execution(execution_id).await
    }

    /// Clear an execution from memory (checkpoint remains in Valkey)
    pub async fn clear_execution(&self, execution_id: &str) -> Result<()> {
        self.active_checkpoints
            .write()
            .await
            .remove(execution_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Valkey to be running
    async fn test_orchestrator_basic() {
        let config = OrchestratorConfig::new("redis://localhost:6379")
            .with_model("gpt-4")
            .with_checkpoint_interval(1);

        let mut plan = ExecutionPlan::new("test-plan");

        plan.add_step(Arc::new(|checkpoint| {
            checkpoint = checkpoint.with_state("step1", serde_json::json!(1));
            Ok(Continuation::Continue)
        }));

        let mut orchestrator = Orchestrator::new(config).await.unwrap();

        // For now, we can't easily register steps after creation
        // This test is a placeholder for when the API is refined
        // let execution_id = orchestrator.start_execution(None).await.unwrap();
        // let result = orchestrator.run(&execution_id).await.unwrap();
        // assert!(result.is_complete());
    }
}
