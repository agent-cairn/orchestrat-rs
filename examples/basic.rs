use orchestrat_rs::{Checkpoint, Orchestrator, OrchestratorConfig};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    // Configure the orchestrator
    let config = OrchestratorConfig::new("redis://localhost:6379")
        .with_model("gpt-4")
        .with_max_retries(3)
        .with_checkpoint_interval(1);

    // Create the orchestrator
    let mut orchestrator = Orchestrator::new(config).await?;

    // Register workflow steps
    orchestrator.register_step(Arc::new(|checkpoint: &mut Checkpoint| {
        println!("Step 1: Initializing workflow");
        *checkpoint = checkpoint
            .with_state("initialized", serde_json::json!(true))
            .with_state("counter", serde_json::json!(0));
        Ok(())
    }));

    orchestrator.register_step(Arc::new(|checkpoint: &mut Checkpoint| {
        println!("Step 2: Processing data");
        let counter = checkpoint.get_state("counter").and_then(|v| v.as_i64()).unwrap_or(0);
        *checkpoint = checkpoint.with_state("counter", serde_json::json!(counter + 1));
        Ok(())
    }));

    orchestrator.register_step(Arc::new(|checkpoint: &mut Checkpoint| {
        println!("Step 3: Finalizing workflow");
        *checkpoint = checkpoint.with_state("completed", serde_json::json!(true));
        Ok(())
    }));

    // Start a new execution
    let execution_id = orchestrator.start_execution(None).await?;
    println!("Started execution: {}", execution_id);

    // Run the workflow
    let result = orchestrator.run(&execution_id).await?;

    println!("\nExecution completed!");
    println!("Final step: {}", result.step_index);
    println!("State: {:#?}", result.state);

    // Demonstrate resumption
    println!("\n--- Resuming execution ---");
    let resumed = orchestrator.get_state(&execution_id).await?;
    println!("Resumed from step: {}", resumed.step_index);

    Ok(())
}
