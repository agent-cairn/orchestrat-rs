# orchestrat-rs

Rust-native LLM orchestration engine with Valkey persistence.

## Architecture

- **LLM Abstraction**: [rig-core](https://github.com/daniel-grignasso/rig) for multi-model LLM support
- **Async Runtime**: Tokio 1.x
- **Persistence**: [valkey-glide](https://github.com/valkey-io/valkey-glide) for Valkey/Redis storage
- **Execution Model**: Continuation-based with automatic checkpointing

## Features

- **Checkpoint/Restore**: Save and resume workflow state at any step
- **Valkey Streams**: Event publishing for external monitoring
- **Error Recovery**: Automatic checkpointing on step failures
- **Configurable**: Retry policies, checkpoint intervals, TTL

## Usage

```rust
use orchestrat_rs::{Orchestrator, OrchestratorConfig, Checkpoint};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = OrchestratorConfig::new("redis://localhost:6379")
        .with_model("gpt-4")
        .with_checkpoint_interval(5);

    let mut orchestrator = Orchestrator::new(config).await?;

    // Register workflow steps
    orchestrator.register_step(Arc::new(|checkpoint: &mut Checkpoint| {
        // Your step logic here
        Ok(())
    }));

    let execution_id = orchestrator.start_execution(None).await?;
    let result = orchestrator.run(&execution_id).await?;

    println!("Execution completed at step {}", result.step_index);
    Ok(())
}
```

## Development

```bash
# Check compilation
cargo check

# Run example (requires Valkey/Redis running)
cargo run --example basic

# Run tests
cargo test
```

## License

Apache-2.0
