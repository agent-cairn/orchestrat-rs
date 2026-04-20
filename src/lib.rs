//! Orchestrat-rs: Rust-native LLM orchestration engine with Valkey persistence
//!
//! This library provides a continuation-based execution engine for LLM workflows,
//! with checkpointing and Valkey Streams persistence.

mod checkpoint;
mod config;
mod error;
mod executor;
mod llm;
mod orchestrator;
mod persistence;

pub use checkpoint::Checkpoint;
pub use config::OrchestratorConfig;
pub use error::{OrchestratError, Result};
pub use executor::{Continuation, ExecutionEngine, ExecutionPlan, Step};
pub use llm::LlmClient;
pub use orchestrator::Orchestrator;
pub use persistence::{Persistence, ValkeyPersistence, ValkeyPersistenceConfig};
