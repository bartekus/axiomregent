# Implementation Plan - Fix Workspace Root Discovery

## Goal
Ensure the MCP server correctly identifies the workspace root (e.g. repo root) even when launched from a subdirectory, instead of defaulting to search roots (e.g. `~/Dev`).

## Proposed Changes

### 1. `src/util/paths.rs`
- Add `discover_workspace_root(start: &Path) -> PathBuf` helper.
- Logic: Walk up from `start` looking for `.axiomregent` or `.git`.

### 2. `src/main.rs`
- Use `discover_workspace_root` starting from `cwd` to determine `run_root`.
- Ensure `.axiomregent` directory exists before creating `encore.lock` inside it.

## Verification Plan
- Add `tests/workspace_discovery_test.rs` with unit tests for:
    - Repo root discovery.
    - Subdirectory start (walk up).
    - Fallback to CWD.
    - Preference for `.axiomregent` over `.git`.
- Run `make check` to ensure no regressions.
