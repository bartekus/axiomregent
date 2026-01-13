# Walkthrough - PR 002: Snapshot & Workspace Tools

## Goal
Integrate `SnapshotTools` and `WorkspaceTools` into the MCP router to expose repository snapshot and workspace management capabilities.

## Changes

### `src/lib.rs`
- Exposed `snapshot` and `workspace` modules.

### `src/router/mod.rs`
- **Dependency Injection**: Updated `Router` to accept `SnapshotTools` and `WorkspaceTools`.
- **Tool Registration**: Added schemas for `snapshot.*` and `workspace.*` tools in `tools/list`.
- **Request Handling**: Implemented dispatch logic in `handle_request` for:
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
- **Error Handling**: 
    - Replaced `?` operator with generic error handling to fix compilation.
    - Added helper functions `handle_tool_result_value`, `handle_tool_result_bool`, and `handle_tool_error` to standardize response formatting.
    - Implemented specific handling for `STALE_LEASE` errors to return the correct JSON-RPC error code and data as per spec.

### `src/main.rs`
- Initialized `StorageConfig`, `Store`, `LeaseStore`.
- Instantiated `SnapshotTools` and `WorkspaceTools`.
- Passed these tools to `Router::new`.

### Tests
- **Updated Integration Tests**: Modified `tests/mcp_tools_test.rs`, `tests/mcp_router_contract_test.rs`, `tests/stale_lease_test.rs`, and `tests/mcp_contract.rs` to match the new `Router` signature (removed future tools like `feature_tools` which are not yet implemented).
- **Golden Test**: Updated `tests/golden/tools_list.json` to reflect the newly added tools.
- **Binary Test**: Updated `tests/test_antigravity_integration.rs` to verify `snapshot.list` exists instead of `antigravity.propose`.

## Verification Results

### Automated Tests
Ran `make check` which includes `cargo fmt`, `cargo check`, `cargo clippy`, and `cargo test`.

```
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
...
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
...
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
...
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
...
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
...
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
...
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

All tests passed, including:
- `test_stale_lease_error_structure`: Confirmed that stale lease errors return the correct custom error structure.
- `test_mcp_tools_list_contract`: Confirmed `tools/list` output matches the golden file.

## Key Decisions
- **Error Handling Helper**: Created `handle_tool_result_*` helpers in `src/router/mod.rs` to reduce code duplication and ensure consistent error formatting, especially for the `STALE_LEASE` case which requires specific JSON structure.
- **Incremental Testing**: Commented out or removed references to future tools (Antigravity, Feature, Xray) in tests to ensure the current PR builds cleanly without implementing out-of-scope features.
