use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for a task
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(String);

impl TaskId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    pub fn from_string(s: String) -> Self {
        Self(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Task state tracking lifecycle
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskState {
    Pending,
    Running,
    Completed,
    Failed(String),
}

/// An LLM task with prompt and state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMTask {
    pub id: TaskId,
    pub prompt: String,
    pub state: TaskState,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub result: Option<String>,
}

impl LLMTask {
    pub fn new(prompt: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: TaskId::new(),
            prompt: prompt.into(),
            state: TaskState::Pending,
            created_at: now,
            updated_at: now,
            result: None,
        }
    }

    pub fn with_id(id: TaskId, prompt: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id,
            prompt: prompt.into(),
            state: TaskState::Pending,
            created_at: now,
            updated_at: now,
            result: None,
        }
    }

    pub fn transition_to(&mut self, new_state: TaskState) {
        self.state = new_state;
        self.updated_at = Utc::now();
    }

    pub fn set_result(&mut self, result: String) {
        self.result = Some(result);
        self.transition_to(TaskState::Completed);
    }

    pub fn fail(&mut self, error: String) {
        self.transition_to(TaskState::Failed(error));
    }
}
