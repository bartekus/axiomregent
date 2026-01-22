# Walkthrough: TS Run Start Integration

## Changes
- **`src/tools/encore_ts/tools.rs`**: Implemented `run_start` orchestrator. It compiles the app, generates supervisor configurations dynamically, spawns the supervisor, and pipes logs.
- **`crates/encore/supervisor` & `tsparser`**: Small refactors to make configuration structs public and serializable.
- **Tests**: Added `test_run_persistence` to `tests/encore_integration.rs`.

## Verification
- `test_run_persistence` passed:
    - App compiled.
    - `supervisor.config.json` generated.
    - `infra.config.json` generated.
    - `supervisor-encore` started.
    - Logs captured.
    - Process stopped cleanly.
