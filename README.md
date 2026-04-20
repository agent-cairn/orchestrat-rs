# orchestrat-rs

Rust-native LLM orchestration engine with Valkey-backed state persistence.

## The Gap

Rust developers building LLM-powered applications face a fragmented landscape:

- **Python-only SDKs**: LangChain, AutoGen, andcrewAI are Python-first with Rust bindings as afterthoughts
- **No Rust-native orchestration**: Existing Rust LLM libraries (rig, kalosm) focus on inference, not workflow orchestration
- **State management left to the developer**: No built-in continuation model, checkpointing, or distributed execution

**orchestrat-rs** fills this gap by providing a production-ready orchestration SDK built for Rust from day one.

## Architecture

```
┌─────────────┐
│   LLMTask   │  Task definition (prompt, state, metadata)
└──────┬──────┘
       │
       ▼
┌─────────────┐     ┌──────────────┐     ┌──────────────┐
│  Executor   │────▶│ ValkeyBackend│────▶│ Valkey Stream│
│             │     │              │     │ (persistence)│
└─────────────┘     └──────────────┘     └──────────────┘
       │
       ▼
┌─────────────┐
│  rig-core   │  LLM integration layer
└─────────────┘
```

### Core Components

- **`LLMTask`**: Task representation with unique ID, prompt, state, and result
- **`TaskState`**: Lifecycle tracking (Pending → Running → Completed/Failed)
- **`Executor`**: Orchestrates task execution with automatic state transitions
- **`ValkeyBackend`**: Durable persistence using Valkey Streams for:
  - Task state storage
  - Time-travel debugging via stream history
  - Consumer group support for distributed processing

### Design Decisions

- **rig-core for LLM integration**: Chosen over llm-chain (abandoned) and kalosm (local-only) for its production-ready client support
- **Tokio 1.x runtime**: Stable, battle-tested async runtime (Tokio 2.x not shipping yet)
- **Valkey Streams persistence**: Enables continuation-based execution with built-in replay and distributed processing
- **Continuation-based model**: Tasks can be paused, resumed, and retried without losing context

## Usage

```rust
use orchestrat_rs::{Executor, LLMTask, ValkeyBackend};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize persistence backend
    let backend = ValkeyBackend::new("redis://localhost".to_string());

    // Create executor
    let executor = Executor::new(backend);

    // Create a task
    let task = LLMTask::new("Explain Rust's ownership model");

    // Save task to persistence (in real implementation)
    // backend.save_task(&task).await?;

    // Execute task
    let result = executor.run(task.id).await?;
    println!("Task result: {:?}", result);

    Ok(())
}
```

See `examples/basic_workflow.rs` for a complete working example.

## Installation

```toml
[dependencies]
orchestrat-rs = "0.1"
```

## Status

🚧 **Early Development** - This is a scaffold with stub implementations. Core functionality is being actively developed:

- [x] Task model and state management
- [x] Executor framework
- [x] Persistence interface design
- [ ] Valkey connection implementation
- [ ] rig-core LLM integration
- [ ] CI/CD pipeline
- [ ] Comprehensive tests
- [ ] Documentation

## Roadmap

### v0.1 (Current)
- Scaffold with core types and interfaces
- Stub implementations for executor and persistence
- Basic CI workflow

### v0.2
- Working Valkey connection
- Task save/load from streams
- Basic rig-core integration

### v0.3
- Full LLM execution via rig-core
- Task retry logic
- Consumer group support for distributed processing

## Contributing

This project is part of the FlowFabric ecosystem. See [agent-cairn](https://github.com/agent-cairn) for related projects.

## License

Apache-2.0
