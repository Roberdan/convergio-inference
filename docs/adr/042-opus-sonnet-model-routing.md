# ADR-042: Opus/Sonnet Model Routing for Agent Execution

## Status: Accepted

## Context

Learning #26 established an "Opus only" policy after a Sonnet agent modified production
auth middleware to bypass authentication — just to make its broken tests pass. This led
to a blanket ban: all spawned agents must use Opus.

However, analysis (2026-04-11) revealed the root cause wasn't Sonnet's capability — it was
**context quality**. The agent received vague instructions ("refactor auth middleware")
and made architectural decisions it shouldn't have. With precise instructions specifying
exact files, function signatures, and patterns to follow, Sonnet executes reliably.

The "Opus only" policy wasted tokens: 100% of tasks ran on the most expensive model,
even mechanical work (CRUD, test writing, file moves) that doesn't require deep reasoning.

## Decision

Route agents to models based on task type, not blanket policy:

| Tier | Model | Use for |
|------|-------|---------|
| t1 | Claude Opus | Architecture, security, planning, review, complex reasoning |
| t2 | Claude Sonnet | Mechanical execution with precise instructions from Opus |
| t3+ | Copilot CLI | Reserved (permission issues pending resolution) |

### Implementation

`spawn_backend.rs`:
```rust
pub fn backend_for_tier(tier: &str, model: Option<&str>) -> SpawnBackend {
    match tier {
        "t1" => SpawnBackend::ClaudeCli { model: "claude-opus-4-6" },
        "t2" => SpawnBackend::ClaudeCli { model: "claude-sonnet-4-6" },
        _ => SpawnBackend::CopilotCli { model: Some("claude-opus-4-6") },
    }
}
```

`spawn_routes.rs`: spawn gate accepts both t1 and t2 for code tasks.

The planner declares `execution_tier` per task. The daemon enforces it.

### Key principle

**Opus thinks, Sonnet builds.** Sonnet receives instructions that specify:
- Exact file paths and function signatures
- Reference patterns from existing code
- What NOT to touch
- Expected output format

Sonnet should never make design decisions. If the task requires judgment, use t1.

## Consequences

- ~60% of tasks (mechanical) can run on Sonnet → estimated +50% throughput
- Learning #26 is superseded, not invalidated — vague instructions + Sonnet still fails
- The planner must be explicit about execution_tier (added to agents/planner.md)
- Copilot CLI deferred: permission issues writing files outside worktrees
