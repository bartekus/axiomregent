# Encore Supervisor MCP Tools Walkthrough

## Summary
Implemented the MCP tools for controlling the Encore Supervisor daemon: `encore.status`, `encore.restart`, and `encore.logs`. These tools enable clients to monitor the supervisor state, trigger restarts, and view process logs.

## Changes

### 1. Supervisor Tools
Implemented `SupervisorTools` in `src/supervisor/tools.rs` exposing:
- `encore.status`: Returns current `state` (Stopped, Starting, Healthy, etc.), PID, and endpoint.
- `encore.restart`: Triggers a restart (Stop -> Start sequence).
- `encore.logs`: Retrieves lines from the ring buffer with `limit` and `offset`.

### 2. Router Integration
- Updated `src/router/mod.rs` to register the new tools.
- Handlers delegate to `SupervisorTools`.

### 3. Application Lifecycle (`main.rs`)
- Initialized `LogBuffer`, `SupervisorHandle`, and `Supervisor` in `src/main.rs`.
- Implemented file locking (`.axiomregent/encore.lock`) to ensure only one "Managed" supervisor instance runs.
- Converted `main` to use `#[tokio::main]` to support the async supervisor task.

### 4. Logic Fixes
- Refactored `Supervisor::run` to eliminate unnecessary inner loops and optimize regex creation.
- Updated `SupervisorHandle` and `SupervisorTools` to handle restarts synchronously via `try_send` to integrate with the synchronous `Router` architecture.

## Verification
- **Unit Tests**: Added `tests/verify_supervisor_tools.rs` to verify tool logic.
- **Integration Tests**: Updated `tests/check_supervisor_integration.rs` and `tests/verify_readiness.rs` to use the new `SupervisorHandle` paradigm and fixed infinite loop/hang issues.
- **Contract Tests**: Updated `tests/mcp_contract.rs` and `tests/golden/tools_list.json` to include the new tool definitions.
- **Build**: `make check` passes successfully.

## Usage
Clients can now call:
```json
{
  "method": "tools/call",
  "params": {
    "name": "encore.status",
    "arguments": {}
  }
}
```
to get the supervisor status.
