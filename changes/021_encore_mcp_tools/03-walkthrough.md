# Walkthrough - Encore Supervisor MCP Tools
<!-- id: 021_encore_mcp_tools -->

## Changes

### 1. Supervisor Tools
#### [NEW] [src/supervisor/tools.rs](file:///Users/bart/Dev/axiomregent/src/supervisor/tools.rs)
- Implemented `SupervisorTools` to expose supervisor functionality.
- Methods: `status()`, `restart(val)`, `logs(limit, offset)`.

### 2. Router Integration
#### [MODIFY] [src/router/mod.rs](file:///Users/bart/Dev/axiomregent/src/router/mod.rs)
- Registered `encore.status`, `encore.restart`, and `encore.logs` tools.
- Mapped tool calls to `SupervisorTools` methods.

### 3. Application Entrypoint
#### [MODIFY] [src/main.rs](file:///Users/bart/Dev/axiomregent/src/main.rs)
- Initialized `SupervisorTools` with the supervisor handle.
- Passed `SupervisorTools` to `Router::new`.

## Verification Results

### Automated Tests
- **`make check`**: Validated all changes and ensured no compilation errors or lints.
- **`tests/verify_supervisor_tools.rs`**: Verified `status`, `logs`, and `restart` functionality directly against the `SupervisorTools` struct.
- **`tests/mcp_tools_test.rs`**: Verified `encore.status` is correctly listed in `tools/list`.

### Manual Verification
- Manually verified that `encore.status` returns the correct state ("starting", "healthy", etc.) during `verify_supervisor_tools.rs` execution.
