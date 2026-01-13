# Antigravity Automation Agent

**Feature ID**: `ANTIGRAVITY_AUTOMATION`
**Protocol**: [`spec/antigravity/protocol.md`](file:///Users/bart/Dev/axiomregent/spec/antigravity/protocol.md)
**Safety Policy**: [`spec/antigravity/safety.md`](file:///Users/bart/Dev/axiomregent/spec/antigravity/safety.md)
**Implementation**: `crates/antigravity`

## Overview
Antigravity enables autonomous agents to safely propose and execute changes within the repository using a "Repo-Native Change Set" protocol.

## Protocol: `changeset_id_v1`
- **Change Set**: A directory in `changes/<id>/` containing all artifacts for a unit of work.
- **Determinism**: The `id` is derived from the `subject`, `repo_key`, and `base_state` (snapshot/worktree hash).

## Artifacts
- **`00-meta.json`**: Canonical identity & immutable context.
- **`00-meta.json`**: Canonical identity & immutable context.
- **`01-implementation_plan.md`**: Reasoning, design, and execution graph.
- **`04-walkthrough.md`**: Proof of execution, including logs and results.
- **`05-status.json`**: Executor-managed state (not written by Agent).

## Tools

### `antigravity.propose`
- **Description**: Agent proposes a change by writing a draft Change Set.
- **Input**: Goal, Subject, Context.
- **Output**: Path to the new Change Set.

### `antigravity.execute`
- **Description**: Executes a validated Change Set.
- **Process**:
  1. **Validation**: integrity (Meta hash match) and structure checks.
  2. **Safety Checks**: Verifies safety tiers and required approvals.
  3. **Execution**: Runs plan steps via `InternalClient` bridging to MCP tools.
  4. **Verification**: Runs Governance (Drift) post-execution.
  5. **Finalization**: Writes `04-walkthrough.md` and updates status.

## Security & Safety

For detailed safety tiers, tool allowlists, and bypass rules, see [Antigravity Safety Policy](file:///Users/bart/Dev/axiomregent/spec/antigravity/safety.md).

### Safety Tiers Summary
- **Tier 1**: Safe to auto-execute.
- **Tier 2**: Requires human approval (`APPROVED` marker) before execution.
- **Tier 3**: Cannot be executed automatically by Antigravity (Human Only).
