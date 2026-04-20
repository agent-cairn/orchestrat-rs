use crate::persistence::ValkeyBackend;
use crate::task::{LLMTask, TaskId};
use crate::TaskState;
use thiserror::Error;
use tracing::{debug, error, info};

/// Executor error types
#[derive(Error, Debug)]
pub enum ExecutorError {
    #[error("Task not found: {0}")]
    TaskNotFound(TaskId),

    #[error("Persistence error: {0}")]
    PersistenceError(String),

    #[error("LLM execution error: {0}")]
    LLMError(String),
}

/// Executor for running LLM tasks
///
/// The executor coordinates task execution with persistence, ensuring
/// that task state is saved before and after execution.
pub struct Executor {
    backend: ValkeyBackend,
}

impl Executor {
    /// Create a new executor with the given Valkey backend
    pub fn new(backend: ValkeyBackend) -> Self {
        Self { backend }
    }

    /// Execute a single task
    ///
    /// This method:
    /// 1. Loads the task from persistence
    /// 2. Transitions to Running state and saves
    /// 3. Executes the LLM prompt (stub for now)
    /// 4. Transitions to Completed/Failed and saves
    pub async fn run(&self, task_id: TaskId) -> Result<LLMTask, ExecutorError> {
        // Load task
        let mut task = self
            .backend
            .load_task(&task_id)
            .await
            .map_err(|e| ExecutorError::PersistenceError(e.to_string()))?
            .ok_or(ExecutorError::TaskNotFound(task_id.clone()))?;

        info!("Starting task execution: {}", task_id);

        // Transition to running
        task.transition_to(TaskState::Running);
        self.backend
            .save_task(&task)
            .await
            .map_err(|e| ExecutorError::PersistenceError(e.to_string()))?;

        // Execute LLM (stub - rig-core integration will go here)
        debug!("Executing LLM prompt for task {}", task_id);
        match self.execute_llm(&task.prompt).await {
            Ok(result) => {
                task.set_result(result);
                info!("Task completed successfully: {}", task_id);
            }
            Err(e) => {
                task.fail(e.to_string());
                error!("Task failed: {} - {}", task_id, e);
            }
        }

        // Save final state
        self.backend
            .save_task(&task)
            .await
            .map_err(|e| ExecutorError::PersistenceError(e.to_string()))?;

        Ok(task)
    }

    /// Execute the LLM prompt (stub implementation)
    ///
    /// TODO: Integrate with rig-core for actual LLM execution
    async fn execute_llm(&self, prompt: &str) -> Result<String, ExecutorError> {
        // Stub: echo the prompt for now
        // In production, this will use rig-core's LLM client
        Ok(format!("LLM response to: {}", prompt))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        // Test executor creation (persistence backend will be tested separately)
        let backend = ValkeyBackend::new("redis://localhost".to_string());
        let executor = Executor::new(backend);
        assert_eq!(executor.backend.connection_url, "redis://localhost");
    }
}
