# PR 04: Log Ring Buffer

## Goal Description
Implement a Ring Buffer to capture and store the most recent output (stdout/stderr) from the Encore process managed by the Supervisor. This ensures that logs are available for inspection even if they are not streamed to a file, and provides a mechanism for "peeking" at the current process state.

## User Review Required
> [!NOTE]
> `Supervisor` struct will be modified to include `log_buffer`. This might affect existing instantiations (currently believed to be only in tests).

## Proposed Changes

### Supervisor Component

#### [NEW] [buffer.rs](file:///Users/bart/Dev/axiomregent/src/supervisor/buffer.rs)
- Define `LogBuffer` struct.
- Use `VecDeque<String>` with a hardcoded (or configurable) capacity (e.g., 1000 lines).
- Thread-safe implementation (or designed to be wrapped in `Arc<Mutex>`).

#### [MODIFY] [mod.rs](file:///Users/bart/Dev/axiomregent/src/supervisor/mod.rs)
- Add `log_buffer: Arc<Mutex<LogBuffer>>` to `Supervisor` struct.
- Update `Supervisor::run` to write captured lines to `log_buffer`.
- Ensure logs are still passed to `log::info!`.

#### [MODIFY] [mod.rs](file:///Users/bart/Dev/axiomregent/src/supervisor/mod.rs) (Modules)
- Export `buffer`.

## Verification Plan

### Automated Tests
- **Unit Tests**: Create `src/supervisor/buffer_test.rs` (or inside `buffer.rs`) to verify `LogBuffer` capacity behavior (overwriting old logs).
- **Integration Test**: Update or create a test in `tests/` that runs the Supervisor and asserts that logs are present in the buffer.
    - Run: `cargo test supervisor::buffer`
    - Run: `cargo test supervisor`

### Manual Verification
- None required as `Supervisor` is not yet fully integrated into the CLI toolchain for manual invocation.
