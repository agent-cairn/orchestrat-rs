use orchestrat_rs::{Executor, LLMTask, TaskState, ValkeyBackend};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for structured logging
    tracing_subscriber::fmt::init();

    // Initialize persistence backend
    // In production, use a real Valkey connection string
    let backend = ValkeyBackend::new("redis://localhost".to_string());

    // Create executor
    let executor = Executor::new(backend);

    // Create a task
    let task = LLMTask::new("Explain Rust's ownership model in simple terms");

    println!("Created task: {} (state: {:?})", task.id, task.state);

    // In a real implementation, you would save the task to persistence first:
    // backend.save_task(&task).await?;

    // Execute the task
    let result = executor.run(task.id.clone()).await?;

    println!("\nTask execution complete:");
    println!("  ID: {}", result.id);
    println!("  State: {:?}", result.state);
    println!("  Prompt: {}", result.prompt);

    if let Some(ref output) = result.result {
        println!("  Result: {}", output);
    }

    if let Some(ref error) = result.error {
        println!("  Error: {}", error);
    }

    // Verify the task is in a terminal state
    assert!(matches!(
        result.state,
        TaskState::Completed | TaskState::Failed
    ));

    Ok(())
}
