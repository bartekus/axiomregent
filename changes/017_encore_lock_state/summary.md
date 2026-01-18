# Change 017: Encore Supervisor Lock & State

## Summary
This change establishes the foundation for the Encore Daemon Supervisor within the MCP server. It includes the lifecycle specification, acceptance criteria, and the core Rust modules for state management and OS-level locking.

## Features
- **Specification**: `spec/core/encore-supervisor.md` defining the state machine and ownership model.
- **Acceptance Tests**: `tests/golden/encore_supervisor_trace.golden` defining expected log prefixes.
- **Locking**: `encore::supervisor::lock` implementing exclusive file locking via `fs2`.
- **State**: `encore::supervisor::state` defining `Ownership`, `State`, and `SupervisorStatus`.

## Files Modified
- `spec/core/encore-supervisor.md` (New)
- `tests/golden/encore_supervisor_trace.golden` (New)
- `crates/encore/supervisor/Cargo.toml` (Modified: added `fs2`, `tempfile`)
- `crates/encore/supervisor/src/lib.rs` (Modified: exposed modules)
- `crates/encore/supervisor/src/lock.rs` (New)
- `crates/encore/supervisor/src/state.rs` (New)
- `crates/encore/supervisor/src/lock_tests.rs` (New)

## Verification
- Unit tests in `lock_tests.rs` pass, verifying:
    - Lock acquisition success.
    - Exclusive locking (second attempt fails).
    - Lock release on drop.
