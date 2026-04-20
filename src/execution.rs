use crate::checkpoint::Checkpoint;
use crate::error::Result;
use std::sync::Arc;

/// A single executable step in the workflow
pub type Step = Arc<dyn Fn(&mut Checkpoint) -> Result<()> + Send + Sync>;

/// Execution engine for running orchestrated workflows
pub struct ExecutionEngine {
    steps: Vec<Step>,
    checkpoint_interval: usize,
}

impl ExecutionEngine {
    /// Create a new execution engine
    pub fn new(checkpoint_interval: usize) -> Self {
        Self {
            steps: Vec::new(),
            checkpoint_interval,
        }
    }

    /// Add a step to the execution plan
    pub fn add_step(&mut self, step: Step) {
        self.steps.push(step);
    }

    /// Execute the workflow from a given checkpoint
    pub async fn execute_from<F>(
        &self,
        mut checkpoint: Checkpoint,
        mut should_checkpoint: F,
    ) -> Result<Checkpoint>
    where
        F: FnMut(&Checkpoint) -> bool,
    {
        let start_index = checkpoint.step_index;
        tracing::info!(
            execution_id = %checkpoint.execution_id,
            start_index,
            total_steps = self.steps.len(),
            "Starting execution"
        );

        for i in start_index..self.steps.len() {
            checkpoint.step_index = i;

            tracing::debug!(
                execution_id = %checkpoint.execution_id,
                step_index = i,
                "Executing step"
            );

            // Execute the step
            if let Err(e) = (self.steps[i])(&mut checkpoint) {
                tracing::error!(
                    execution_id = %checkpoint.execution_id,
                    step_index = i,
                    error = %e,
                    "Step failed, saving error checkpoint"
                );

                // Save error state to checkpoint
                checkpoint = checkpoint.with_state(
                    "error",
                    serde_json::json!({
                        "step": i,
                        "message": e.to_string(),
                    }),
                );
                return Ok(checkpoint);
            }

            // Checkpoint if needed
            if should_checkpoint(&checkpoint) {
                tracing::debug!(
                    execution_id = %checkpoint.execution_id,
                    step_index = i,
                    "Checkpointing"
                );
                // Note: Actual persistence is handled by the caller
            }
        }

        checkpoint.step_index = self.steps.len();
        tracing::info!(
            execution_id = %checkpoint.execution_id,
            "Execution completed successfully"
        );

        Ok(checkpoint)
    }

    /// Get the total number of steps
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execution_basic() {
        let mut engine = ExecutionEngine::new(10);

        engine.add_step(Arc::new(|checkpoint| {
            checkpoint = checkpoint.with_state("step1", serde_json::json!("done"));
            Ok(())
        }));

        engine.add_step(Arc::new(|checkpoint| {
            checkpoint = checkpoint.with_state("step2", serde_json::json!("done"));
            Ok(())
        }));

        let checkpoint = Checkpoint::new("test-exec", 0).with_total_steps(2);
        let result = engine
            .execute_from(checkpoint, |_| false)
            .await
            .unwrap();

        assert_eq!(result.step_index, 2);
        assert_eq!(result.get_state("step1"), Some(&serde_json::json!("done")));
        assert_eq!(result.get_state("step2"), Some(&serde_json::json!("done")));
    }
}
