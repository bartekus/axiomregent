# PR 002: Snapshot & Workspace Tools

## Goal
Integrate the `SnapshotTools` and `WorkspaceTools` into the Core MCP Router to enable:
1.  **Snapshot Mode**: Reading from immutable snapshots.
2.  **Worktree Mode**: Reading and mutating the live worktree with `Lease` safety.

## Proposed Changes
We will modify `src/main.rs` to initialize the necessary stores (`LeaseStore`, `Store`) and tools, and pass them to the `Router`.
We will modify `src/router/mod.rs` to register and dispatch these tools.

### Components

#### [MODIFY] [src/main.rs](file:///Users/bart/Dev/axiomregent/src/main.rs)
- Initialize `StorageConfig::default()`.
- Initialize `Store` and `LeaseStore`.
- Initialize `SnapshotTools` and `WorkspaceTools`.
- Update `Router::new()` call.

#### [MODIFY] [src/router/mod.rs](file:///Users/bart/Dev/axiomregent/src/router/mod.rs)
- Update `Router` struct to hold `SnapshotTools` and `WorkspaceTools`.
- Update `Router::new()` signature.
- Update `handle_request` "tools/list" to include:
    - `snapshot.list`
    - `snapshot.create`
    - `snapshot.read` (mapped to `snapshot_file`)
    - `snapshot.grep`
    - `snapshot.diff`
    - `snapshot.changes`
    - `snapshot.export`
    - `snapshot.info`
    - `workspace.write_file`
    - `workspace.delete`
    - `workspace.apply_patch`
- Update `handle_request` "tools/call" to dispatch to these tools.

## Verification Plan
### Automated Tests
- Run `make check` to ensure compilation and linting passes.
- We will rely on existing unit tests in `snapshot/tools.rs` and `workspace/mod.rs` for logic correctness.
- We will verify integration by compiling.

### Manual Verification
- We can inspect `tools/list` output via a mock client or golden test if applicable, but for this PR, successful compilation and existing unit tests are the primary gate.
