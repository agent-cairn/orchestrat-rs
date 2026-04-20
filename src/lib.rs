//! Orchestrat-rs: Rust-native LLM orchestration engine with Valkey persistence
//!
//! This library provides a continuation-based execution engine for LLM workflows,
//! with checkpointing and Valkey Streams persistence.

mod checkpoint;
mod config;
mod error;
mod execution;
mod orchestrator;
mod persistence;

pub use checkpoint::Checkpoint;
pub use config::OrchestratorConfig;
pub use error::{OrchestratError, Result};
pub use execution::ExecutionEngine;
pub use orchestrator::Orchestrator;
pub use persistence::{ValkeyPersistence, ValkeyPersistenceConfig};
