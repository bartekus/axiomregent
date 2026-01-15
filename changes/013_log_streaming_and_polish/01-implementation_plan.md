# 013 Log Streaming & Polish

This change enhances the `run` tools to support efficient log retrieval (streaming via polling) and richer status reporting.

## User Review Required

> [!NOTE]
> `run.logs` now supports `offset` and `limit` to allow clients to stream logs by polling, rather than fetching the entire file every time.

## Proposed Changes

### `src/run_tools.rs`
- **Structure Update**: Update `RunContext` to include `start_time` and `end_time`.
- **`logs` Tool**:
    - Update signature: `logs(run_id, offset: Option<u64>, limit: Option<u64>)`.
    - Implement seeking and partial reading of the log file.
- **`status` Tool**:
    - Return a JSON object (serialized as string or just Map) containing:
        - `status`: running/completed/failed
        - `start_time`: ISO8601 string
        - `end_time`: ISO8601 string (if finished)
        - `exit_code`: if available (might need to update `run` crate or infer from result).

### `src/router/mod.rs`
- Update `run.logs` tool definition to include `offset` and `limit` arguments.

## Verification Plan

### Automated Tests
- **Integration Tests**:
    - Test `run.logs` with varying offsets and limits to verify data integrity.
    - Test `run.status` for presence of timestamps.

### Manual Verification
- N/A (Relied on automated tests).
