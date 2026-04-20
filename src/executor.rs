use crate::checkpoint::Checkpoint;
use crate::error::Result;
use std::sync::Arc;

/// A single executable step in the workflow
pub type Step = Arc<dyn Fn(&mut Checkpoint) -> Result<()> + Send + Sync>;

/// Continuation-based execution state - allows pausing and resuming execution
#[derive(Debug, Clone)]
pub enum Continuation {
    /// Continue executing next step
    Continue,
    /// Pause execution and wait for external signal
    Pause,
    /// Branch to a specific step index
    Jump(usize),
    /// Complete execution successfully
    Complete,
    /// Fail with error
    Fail(String),
}

/// A step that returns a continuation for flow control
pub type ContinuationStep = Arc<dyn Fn(&mut Checkpoint) -> Result<Continuation> + Send + Sync>;

/// Execution plan with steps and flow control
pub struct ExecutionPlan {
    steps: Vec<ContinuationStep>,
    name: String,
}

impl ExecutionPlan {
    /// Create a new execution plan
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            steps: Vec::new(),
            name: name.into(),
        }
    }

    /// Add a step to the plan
    pub fn add_step(&mut self, step: ContinuationStep) {
        self.steps.push(step);
    }

    /// Get the number of steps
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Get the plan name
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Execution engine for running orchestrated workflows with continuation-based control
pub struct ExecutionEngine {
    plan: ExecutionPlan,
    checkpoint_interval: usize,
}

impl ExecutionEngine {
    /// Create a new execution engine
    pub fn new(plan: ExecutionPlan) -> Self {
        Self {
            plan,
            checkpoint_interval: 10,
        }
    }

    /// Set checkpoint interval
    pub fn with_checkpoint_interval(mut self, interval: usize) -> Self {
        self.checkpoint_interval = interval;
        self
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
        let mut current_index = start_index;

        tracing::info!(
            execution_id = %checkpoint.execution_id,
            plan = %self.plan.name(),
            start_index,
            total_steps = self.plan.step_count(),
            "Starting execution"
        );

        while current_index < self.plan.step_count() {
            checkpoint.step_index = current_index;

            tracing::debug!(
                execution_id = %checkpoint.execution_id,
                step_index = current_index,
                "Executing step"
            );

            // Execute the step and get continuation
            let continuation = (self.plan.steps[current_index])(&mut checkpoint)?;

            match continuation {
                Continuation::Continue => {
                    current_index += 1;
                }
                Continuation::Pause => {
                    tracing::debug!(
                        execution_id = %checkpoint.execution_id,
                        step_index = current_index,
                        "Execution paused"
                    );
                    break;
                }
                Continuation::Jump(target_index) => {
                    if target_index < self.plan.step_count() {
                        tracing::debug!(
                            execution_id = %checkpoint.execution_id,
                            from_step = current_index,
                            to_step = target_index,
                            "Branching execution"
                        );
                        current_index = target_index;
                    } else {
                        return Err(crate::error::OrchestratError::Execution(format!(
                            "Jump target {} out of bounds (total steps: {})",
                            target_index,
                            self.plan.step_count()
                        )));
                    }
                }
                Continuation::Complete => {
                    checkpoint.step_index = self.plan.step_count();
                    tracing::info!(
                        execution_id = %checkpoint.execution_id,
                        "Execution completed early via Continue::Complete"
                    );
                    return Ok(checkpoint);
                }
                Continuation::Fail(error) => {
                    tracing::error!(
                        execution_id = %checkpoint.execution_id,
                        step_index = current_index,
                        error = %error,
                        "Execution failed via Continue::Fail"
                    );

                    // Save error state to checkpoint
                    checkpoint = checkpoint.with_state(
                        "error",
                        serde_json::json!({
                            "step": current_index,
                            "message": error,
                        }),
                    );
                    return Ok(checkpoint);
                }
            }

            // Checkpoint if needed
            if should_checkpoint(&checkpoint) {
                tracing::debug!(
                    execution_id = %checkpoint.execution_id,
                    step_index = current_index,
                    "Checkpointing"
                );
                // Note: Actual persistence is handled by the caller
            }
        }

        if current_index >= self.plan.step_count() {
            checkpoint.step_index = self.plan.step_count();
            tracing::info!(
                execution_id = %checkpoint.execution_id,
                "Execution completed successfully"
            );
        }

        Ok(checkpoint)
    }

    /// Get the execution plan
    pub fn plan(&self) -> &ExecutionPlan {
        &self.plan
    }

    /// Get the total number of steps
    pub fn step_count(&self) -> usize {
        self.plan.step_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execution_basic() {
        let mut plan = ExecutionPlan::new("test-plan");

        plan.add_step(Arc::new(|checkpoint| {
            checkpoint = checkpoint.with_state("step1", serde_json::json!("done"));
            Ok(Continuation::Continue)
        }));

        plan.add_step(Arc::new(|checkpoint| {
            checkpoint = checkpoint.with_state("step2", serde_json::json!("done"));
            Ok(Continuation::Continue)
        }));

        let engine = ExecutionEngine::new(plan);
        let checkpoint = Checkpoint::new("test-exec", 0).with_total_steps(2);
        let result = engine
            .execute_from(checkpoint, |_| false)
            .await
            .unwrap();

        assert_eq!(result.step_index, 2);
        assert_eq!(result.get_state("step1"), Some(&serde_json::json!("done")));
        assert_eq!(result.get_state("step2"), Some(&serde_json::json!("done")));
    }

    #[tokio::test]
    async fn test_continuation_jump() {
        let mut plan = ExecutionPlan::new("jump-test");

        plan.add_step(Arc::new(|checkpoint| {
            checkpoint = checkpoint.with_state("step1", serde_json::json!("executed"));
            Ok(Continuation::Jump(2)) // Skip step 2
        }));

        plan.add_step(Arc::new(|checkpoint| {
            checkpoint = checkpoint.with_state("step2", serde_json::json!("should-not-run"));
            Ok(Continuation::Continue)
        }));

        plan.add_step(Arc::new(|checkpoint| {
            checkpoint = checkpoint.with_state("step3", serde_json::json!("executed"));
            Ok(Continuation::Continue)
        }));

        let engine = ExecutionEngine::new(plan);
        let checkpoint = Checkpoint::new("test-jump", 0).with_total_steps(3);
        let result = engine
            .execute_from(checkpoint, |_| false)
            .await
            .unwrap();

        assert_eq!(result.step_index, 3);
        assert_eq!(result.get_state("step1"), Some(&serde_json::json!("executed")));
        assert!(result.get_state("step2").is_none());
        assert_eq!(result.get_state("step3"), Some(&serde_json::json!("executed")));
    }

    #[tokio::test]
    async fn test_continuation_pause() {
        let mut plan = ExecutionPlan::new("pause-test");

        plan.add_step(Arc::new(|checkpoint| {
            checkpoint = checkpoint.with_state("step1", serde_json::json!("done"));
            Ok(Continuation::Pause)
        }));

        plan.add_step(Arc::new(|checkpoint| {
            checkpoint = checkpoint.with_state("step2", serde_json::json!("should-not-run"));
            Ok(Continuation::Continue)
        }));

        let engine = ExecutionEngine::new(plan);
        let checkpoint = Checkpoint::new("test-pause", 0).with_total_steps(2);
        let result = engine
            .execute_from(checkpoint, |_| false)
            .await
            .unwrap();

        // Should pause after step 0
        assert_eq!(result.step_index, 0);
        assert_eq!(result.get_state("step1"), Some(&serde_json::json!("done")));
        assert!(result.get_state("step2").is_none());
    }

    #[tokio::test]
    async fn test_continuation_fail() {
        let mut plan = ExecutionPlan::new("fail-test");

        plan.add_step(Arc::new(|checkpoint| {
            Ok(Continuation::Fail("intentional test failure".to_string()))
        }));

        let engine = ExecutionEngine::new(plan);
        let checkpoint = Checkpoint::new("test-fail", 0).with_total_steps(1);
        let result = engine
            .execute_from(checkpoint, |_| false)
            .await
            .unwrap();

        assert_eq!(result.step_index, 0);
        assert!(result.get_state("error").is_some());
    }
}
