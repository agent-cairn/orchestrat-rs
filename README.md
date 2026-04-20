# orchestrat-rs

Rust-native LLM orchestration engine with Valkey persistence and WASM sandbox isolation.

## Overview

`orchestrat-rs` provides a continuation-based execution engine for building complex LLM workflows with:

- **Continuation-based execution**: Pause, resume, and branch execution flows
- **Valkey persistence**: Checkpoint and restore execution state with Valkey Streams
- **WASM sandboxing**: Secure isolation for untrusted code execution
- **Rig-core integration**: LLM abstraction with retry and streaming support
- **Tokio async runtime**: High-performance async I/O

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Orchestrator                           │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────────┐  │
│  │ Execution    │  │ Persistence  │  │  WASM Sandbox   │  │
│  │   Engine     │◄─┤   Layer      │  │     (Wasmtime)  │  │
│  └──────┬───────┘  └──────┬───────┘  └────────┬────────┘  │
│         │                  │                    │            │
│         │         ┌────────▼────────┐          │            │
│         └─────────┤  Valkey/Glide   │◄─────────┘            │
│                   │  (Persistence)  │                       │
│                   └─────────────────┘                       │
└─────────────────────────────────────────────────────────────┘
                           │
                    ┌──────▼──────┐
                    │  rig-core   │
                    │   (LLM)     │
                    └─────────────┘
```

### Key Components

- **Executor (`executor.rs`)**: Continuation-based execution engine with flow control
- **LLM Integration (`llm.rs`)**: Rig-core wrapper with retry and streaming
- **Persistence (`persistence.rs`)**: Valkey-backed checkpoint and event streaming
- **Checkpoint (`checkpoint.rs`)**: Execution state serialization
- **Sandbox (`sandbox.rs`)**: Wasmtime-based isolation for untrusted code
- **Config (`config.rs`)**: Centralized configuration management
- **Error (`error.rs`)**: Comprehensive error types with `thiserror`

## Usage

### Basic Example

```rust
use orchestrat_rs::{
    executor::{Continuation, ExecutionEngine, ExecutionPlan},
    checkpoint::Checkpoint,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an execution plan
    let mut plan = ExecutionPlan::new("my-workflow");

    // Add steps
    plan.add_step(Arc::new(|checkpoint| {
        checkpoint = checkpoint.with_state("step1", serde_json::json!("done"));
        Ok(Continuation::Continue)
    }));

    plan.add_step(Arc::new(|checkpoint| {
        checkpoint = checkpoint.with_state("step2", serde_json::json!("done"));
        Ok(Continuation::Complete)
    }));

    // Execute
    let engine = ExecutionEngine::new(plan);
    let checkpoint = Checkpoint::new("exec-123", 0).with_total_steps(2);
    let result = engine.execute_from(checkpoint, |_| false).await?;

    println!("Execution completed at step {}", result.step_index);
    Ok(())
}
```

### With LLM Integration

```rust
use orchestrat_rs::{llm::{LlmClient, LlmConfig}, executor::{Continuation, ExecutionEngine, ExecutionPlan}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create LLM client
    let llm_config = LlmConfig::new("gpt-4")
        .with_max_tokens(2048)
        .with_temperature(0.7);

    let client = LlmClient::new(llm_config)?;

    // Create execution plan with LLM step
    let mut plan = ExecutionPlan::new("llm-workflow");

    plan.add_step(Arc::new(|checkpoint| {
        // In a real scenario, you'd call the LLM here
        checkpoint = checkpoint.with_state("llm_result", serde_json::json!("response"));
        Ok(Continuation::Continue)
    }));

    // Execute
    let engine = ExecutionEngine::new(plan);
    // ... execution logic

    Ok(())
}
```

### With Valkey Persistence

```rust
use orchestrat_rs::{
    orchestrator::Orchestrator,
    config::OrchestratorConfig,
    executor::{Continuation, ExecutionEngine, ExecutionPlan},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure orchestrator with Valkey
    let config = OrchestratorConfig::new("redis://localhost:6379")
        .with_model("gpt-4")
        .with_checkpoint_interval(5);

    let orchestrator = Orchestrator::new(config).await?;

    // Start execution
    let execution_id = orchestrator.start_execution(None).await?;

    // ... execution logic

    // Resume execution
    let state = orchestrator.resume_execution(&execution_id).await?;

    Ok(())
}
```

### WASM Sandbox Execution

```rust
use orchestrat_rs::sandbox::{Sandbox, SandboxConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create sandbox with security constraints
    let config = SandboxConfig::new()
        .with_max_execution_time(5000)
        .with_max_memory(64)
        .with_network(false);

    let sandbox = Sandbox::new(config)?;

    // Execute WASM bytes
    let wasm_bytes = include_bytes!("../example.wasm");
    let result = sandbox.execute(wasm_bytes, "main", &[]).await?;

    if result.success {
        println!("Output: {}", result.output);
    } else {
        println!("Error: {}", result.exit_status);
    }

    Ok(())
}
```

## Configuration

### OrchestratorConfig

```rust
use orchestrat_rs::config::OrchestratorConfig;

let config = OrchestratorConfig::new("redis://localhost:6379")
    .with_model("gpt-4")                    // LLM model
    .with_max_retries(3)                   // Retry attempts
    .with_checkpoint_interval(10)          // Steps between checkpoints
    .with_persistence(true);               // Enable/disable persistence
```

### LlmConfig

```rust
use orchestrat_rs::llm::LlmConfig;
use std::time::Duration;

let config = LlmConfig::new("gpt-4")
    .with_api_key("sk-...")                // Optional (uses env var)
    .with_max_tokens(2048)
    .with_temperature(0.7)
    .with_max_retries(3)
    .with_retry_delay(Duration::from_millis(1000));
```

### SandboxConfig

```rust
use orchestrat_rs::sandbox::SandboxConfig;

let config = SandboxConfig::new()
    .with_max_execution_time(5000)         // 5 seconds
    .with_max_memory(64)                   // 64 MB
    .with_network(false)                   // Disable network
    .with_filesystem(false)                // Disable filesystem
    .with_env_var("KEY", "value");         // Environment variables
```

## Continuation-Based Execution

The execution model uses continuations for fine-grained flow control:

```rust
use orchestrat_rs::executor::Continuation;

// Continue to next step
Continuation::Continue

// Pause execution (save checkpoint)
Continuation::Pause

// Jump to specific step index
Continuation::Jump(5)

// Complete execution early
Continuation::Complete

// Fail with error
Continuation::Fail("something went wrong".to_string())
```

## Persistence with Valkey

Checkpoints are persisted to Valkey with:

- **Checkpoint storage**: `SETEX` with configurable TTL
- **Event streaming**: `XADD` to Valkey Streams (or `LPUSH` fallback)
- **Key prefix**: `orchestrat:checkpoint:{execution_id}`
- **Stream name**: `orchestrat:events`

Example events:
```json
{
  "type": "checkpoint_saved",
  "execution_id": "exec-123",
  "step_index": 5,
  "timestamp": 1234567890
}
```

## Security Notes

### WASM Sandbox

- **Memory limits**: Enforced via Wasmtime `StoreLimits`
- **Timeout**: Hard timeout on execution time
- **Network access**: Disabled by default
- **Filesystem access**: Disabled by default
- **CVE-2026-34987/34971**: Protected by Wasmtime ≥44.0.0

### Valkey Security

- **TTL**: Checkpoints expire after configured time (default: 24h)
- **Connection strings**: Use environment variables, never hardcode
- **Access control**: Follow Valkey ACL best practices

## Dependencies

- **rig-core** (0.5): LLM abstraction and orchestration primitives
- **tokio** (1.x): Async runtime (Tokio 2.x not shipping yet)
- **valkey-glide** (2.0): Valkey client library
- **wasmtime** (44+): WASM runtime with security fixes
- **serde/serde_json**: Serialization
- **thiserror**: Error handling
- **tracing**: Structured logging

## Development

### Running Tests

```bash
# All tests (requires Valkey for integration tests)
cargo test

# Skip integration tests
cargo test -- --skip test_orchestrator_basic --skip test_persistence_basic

# With logs
RUST_LOG=debug cargo test
```

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Check without building
cargo check
```

## License

Apache-2.0

## Contributing

Contributions welcome! Please read the contributing guidelines before submitting PRs.

## Roadmap

- [ ] Streaming LLM responses
- [ ] Distributed execution with Valkey Pub/Sub
- [ ] Advanced checkpoint strategies (incremental, differential)
- [ ] Metrics and observability integration
- [ ] Workflow DSL / YAML configuration
- [ ] Multi-LLM routing and fallback
