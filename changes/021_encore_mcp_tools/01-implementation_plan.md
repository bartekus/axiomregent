# Implementation Plan - Encore Supervisor MCP Tools

## Goal
Expose the Encore Supervisor functionality via MCP tools (`encore.status`, `encore.restart`, `encore.logs`) to allow clients to monitor and control the Encore daemon.

## User Review Required
> [!IMPORTANT]
> This change introduces persistent state management for the Supervisor in `main.rs`. Ensure `AXIOM_ENCORE_MODE` env var is handled correctly during server startup.

## Proposed Changes

### Supervisor Core
#### [MODIFY] [src/supervisor/mod.rs](file:///Users/bart/Dev/axiomregent/src/supervisor/mod.rs)
- Introduce `SupervisorState` enum (Stopped, Starting, Healthy, Unhealthy, Backoff, Fatal).
- Update `Supervisor` to hold `Arc<RwLock<SupervisorState>>`.
- Implement `SupervisorHandle` to allow external tools to signal restart or query status.

### Tools Implementation
#### [NEW] [src/supervisor/tools.rs](file:///Users/bart/Dev/axiomregent/src/supervisor/tools.rs)
- Implement `SupervisorTools` struct.
- Implement `encore_status`, `encore_restart`, `encore_logs` methods.

### Router Integration
#### [MODIFY] [src/router/mod.rs](file:///Users/bart/Dev/axiomregent/src/router/mod.rs)
- Register `encore.status`, `encore.restart`, `encore.logs`.
- Map tool calls to `SupervisorTools`.

### Application Entrypoint
#### [MODIFY] [src/main.rs](file:///Users/bart/Dev/axiomregent/src/main.rs)
- Initialize `Supervisor` and `SupervisorTools`.
- Spawn Supervisor background task.

## Verification Plan

### Automated Tests
- `tests/mcp_tools_test.rs`: Add test cases for `encore.status` (check initial state) and `encore.logs`.
- `tests/golden/encore_supervisor_tools.golden`: Verify tool output schemas.

### Manual Verification
- Run `cargo run` and use `mcp-inspector` (if available) or `curl` (via helper) to call `encore.status`.
