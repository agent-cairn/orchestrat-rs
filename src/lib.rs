//! Orchestrat-rs: Rust-native LLM orchestration engine
//!
//! This crate provides a continuation-based execution model for LLM workflows
//! with Valkey-backed state persistence. It fills the gap for Rust developers
//! who need a production-ready orchestration SDK without Python dependencies.

pub mod executor;
pub mod persistence;
pub mod state;
pub mod task;

pub use executor::Executor;
pub use persistence::ValkeyBackend;
pub use state::TaskState;
pub use task::{LLMTask, TaskId};
