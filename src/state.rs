use serde::{Deserialize, Serialize};

/// Task state tracking lifecycle
///
/// Represents the current state of a task in the execution pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskState {
    /// Task has been created but not yet started
    Pending,
    /// Task is currently executing
    Running,
    /// Task completed successfully
    Completed,
    /// Task failed with an error message
    Failed(String),
}

impl TaskState {
    /// Check if the task is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, TaskState::Completed | TaskState::Failed(_))
    }

    /// Check if the task is currently running
    pub fn is_running(&self) -> bool {
        matches!(self, TaskState::Running)
    }

    /// Check if the task has not started yet
    pub fn is_pending(&self) -> bool {
        matches!(self, TaskState::Pending)
    }
}
