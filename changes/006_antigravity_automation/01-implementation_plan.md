# Implementation Plan - PR 006: Antigravity Automation

Integrate `antigravity` crate capabilities into the main AxiomRegent MCP server by exposing `antigravity.propose` and `antigravity.execute` tools.

## Proposed Changes

### Core System
#### [MODIFY] [src/lib.rs](file:///Users/bart/Dev/axiomregent/src/lib.rs)
- Uncomment `pub mod antigravity_tools;`
- Uncomment `pub mod internal_client;`

#### [MODIFY] [src/router/mod.rs](file:///Users/bart/Dev/axiomregent/src/router/mod.rs)
- Import `crate::antigravity_tools::AntigravityTools`.
- Add `antigravity_tools: Arc<AntigravityTools>` to `Router` struct.
- Update `Router::new` signature to accept `antigravity_tools`.
- Update `tools/list` to include:
    - `antigravity.propose` (Schema: `AgentConfig`)
    - `antigravity.execute` (Schema: `changeset_id`, `repo_root`)
- Update `tools/call` to handle `antigravity.propose` and `antigravity.execute` by delegating to `self.antigravity_tools`.

### Application Entry Point
#### [MODIFY] [src/main.rs](file:///Users/bart/Dev/axiomregent/src/main.rs)
- Instantiate `AntigravityTools` using `workspace_tools`, `snapshot_tools`, and `featuregraph_tools`.
- Pass `antigravity_tools` to `Router::new`.

### Tests
#### [MODIFY] [tests/mcp_tools_test.rs](file:///Users/bart/Dev/axiomregent/tests/mcp_tools_test.rs)
- Update `Router::new` calls to pass `AntigravityTools`.
- Add a test case to verify `antigravity.propose` appears in `tools/list`.

#### [MODIFY] [tests/mcp_router_contract_test.rs](file:///Users/bart/Dev/axiomregent/tests/mcp_router_contract_test.rs)
- Update `Router::new` calls to pass `AntigravityTools`.

## Verification Plan

### Automated Tests
- Run `make check` to ensure compilation and all tests pass.
- `cargo test --test mcp_tools_test` to verify specifically the new tool registration.

### Manual Verification
- None required (covered by automated tests and compiler checks).
