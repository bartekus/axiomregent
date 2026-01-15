# AxiomRegent Run CLI Skills

**Feature ID**: `AXIOMREGENT_RUN_SKILLS`
**Implementation**: `crates/run/src/skills`

## Overview
The `axiomregent run` CLI executes high-level "skills" (workflows) that abstract complex build/test operations. These skills are often ported from other languages (e.g., Go) to native Rust.

## Available Skills

### `test:go`
- **Description**: Runs Go tests for the repository.
- **Command**: `go test ./...`
- **Implementation**: `crates/run/src/skills/test_go.rs`

### `test:build`
- **Description**: Verifies the build integrity.
- **Implementation**: `crates/run/src/skills/test_build.rs`

### `lint:gofumpt`
- **Description**: Checks for code formatting issues using `gofumpt`.
- **Implementation**: `crates/run/src/skills/lint_gofumpt.rs`

## Execution Model
- **Command**: `axiomregent run <skill_id> [flags]`
- **Flags**:
    - `--json`: Output results in JSON.
    - `--fail-on-warning`: Fail if warnings occur.
- **State Management**: Persists run results to `.axiomregent/run` state directory.
- **Determinism**: Execution order is stable (lexicographic).

## MCP Integration & Observability
The `run` crate capabilities are exposed via the MCP Router (`RunTools`), providing:
- **Asynchronous Execution**: Skills run in background threads.
- **Rich Status Reporting**: Status includes `running`, `completed` (with exit code 0), or `failed` (non-zero), along with `start_time` and `end_time` timestamps.
- **Log Streaming**: Run logs are persisted to `.axiomregent/run/logs/<run_id>.log` and can be streamed via `run.logs` using `offset` and `limit` parameters for real-time monitoring.
