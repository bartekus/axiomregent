# 012 Env, Run Lifecycle & Logs

This change implements the necessary infrastructure to manage the lifecycle of `run` skills, inject environment variables, and capture logs, exposing these capabilities via the MCP Router.

## User Review Required

> [!IMPORTANT]
> The `run` crate will be refactored from a pure binary to a library + binary structure to allow embedding the runner logic into the MCP server.

## Proposed Changes

### `crates/run` Refactoring
- Move core logic from `src/main.rs` to `src/lib.rs`.
- Ensure `Runner`, `RunConfig`, `StateStore`, etc., are public.
- Update `src/main.rs` to use the library.

### `crates/run` Enhancements
- **Env Support**: Add `env: HashMap<String, String>` to `RunConfig`. Update skills (e.g., `LegacySkill`) to apply these to child processes.
- **Log Capture**: Update `Runner` to accept a `Box<dyn Write + Send>` (or generic writer) for output, allowing redirection of stdout/stderr to a file.

### Root `Cargo.toml`
- Add `run` to `[dependencies]` similar to `featuregraph` etc.

### MCP Tools Implementation (`src/tools/run.rs` & `src/router/mod.rs`)
- Register new tools:
    - `run.execute(skill: String, env: Option<Map<String, String>>)` -> `run_id`
    - `run.status(run_id: String)` -> `status` (running/completed/failed)
    - `run.logs(run_id: String)` -> `content`
- Implement `RunTools` (or `RunService`):
    - Manages background threads for execution.
    - Stores active run handles in memory (`Arc<Mutex<HashMap<String, RunContext>>>`).
    - Writes logs to `.axiomregent/run/logs/<run_id>.log`.

## Verification Plan

### Automated Tests
- **Unit Tests**:
    - Test `Runner` with captured writer to ensure output is written.
    - Test `Runner` with Env vars.
- **Integration Tests**:
    - Test `run.execute` via Router, poll status, check logs.

### Manual Verification
- Run `run.execute` with a simple skill (e.g. `test:build`).
- Verify log file creation.
- Verify status updates.
