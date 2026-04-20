# PR Creation Instructions

## Automated PR Creation (Preferred)

Configure a GitHub Personal Access Token and the PR can be created automatically:

```bash
# 1. Create token at: https://github.com/settings/tokens
#    - Scopes needed: repo (full control of private repos)
#
# 2. Set environment variable:
#    export GH_TOKEN="your_token_here"
#
# 3. Run from orchestrat-rs directory:
#    gh pr create \
#      --base scaffold-llm-orchestrator \
#      --title "Scaffold: Rust-native LLM orchestration engine" \
#      --body-file pr_body.txt \
#      --draft
```

## Manual PR Creation (Alternative)

If automated creation isn't possible:

1. Navigate to: https://github.com/agent-cairn/orchestrat-rs
2. Click "Pull requests" → "New pull request"
3. Select branches:
   - base: `scaffold-llm-orchestrator` (default branch)
   - compare: `feature-initial-scaffold`
4. Click "Create pull request"
5. Set title: `Scaffold: Rust-native LLM orchestration engine`
6. Paste the PR body below
7. Click "Create draft pull request"

---

## PR Body

```markdown
## Scaffold: Rust-native LLM orchestration engine

This PR establishes the foundational structure for `orchestrat-rs`, a Rust-native LLM orchestration engine with Valkey-backed persistence.

### What's Included

**Core files (8/9 complete):**
- ✅ `Cargo.toml` - Dependencies: tokio 1.42, rig-core 0.5, valkey-glide (git), serde, uuid, chrono
- ✅ `src/lib.rs` - Module declarations and public API surface
- ✅ `src/task.rs` - `LLMTask` struct with ID, prompt, state, metadata
- ✅ `src/state.rs` - `TaskState` enum (Pending → Running → Completed/Failed)
- ✅ `src/executor.rs` - `Executor` framework with `run()` method
- ✅ `src/persistence.rs` - Persistence trait + `ValkeyBackend` stub
- ✅ `README.md` - Architecture diagram, usage example, roadmap
- ✅ `examples/basic_workflow.rs` - Working example showing executor flow

**Known gaps:**
- ⚠️ `.github/workflows/ci.yml` - Blocked by OAuth `workflow` scope limitation. See CI_WORKFLOW_INSTRUCTIONS.md for manual setup.
- 📄 `PR_CREATION.md` - This file (can be removed after PR is created)

### Architecture

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

### Design Decisions

- **rig-core** for LLM integration (production-ready client support)
- **Tokio 1.x** runtime (stable, battle-tested)
- **Valkey Streams** for continuation-based execution with time-travel debugging
- **Continuation model** enables pause/resume/retry without context loss

### Next Steps (v0.2)

After merge, prioritize:
1. Add CI workflow manually (see CI_WORKFLOW_INSTRUCTIONS.md)
2. Working Valkey connection implementation
3. Task save/load from Valkey Streams
4. rig-core LLM integration in executor

### Repository Notes

- Default branch: `scaffold-llm-orchestrator` (not `main`)
- All scaffold code is stub implementations ready for real logic
- CI workflow scope issue documented in CI_WORKFLOW_INSTRUCTIONS.md
```
