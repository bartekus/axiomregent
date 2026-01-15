# PR 010: Encore TS Integration Skeleton

## Description
Implements the foundational skeleton for the Encore TS toolchain integration. This includes:
-   **Directory Structure**: Added `src/tools/encore_ts/` with submodules for `env`, `parse`, `projection`, `run`, `schemas`, `state`, and `tools`.
-   **Tool Registration**: Registered `encore.ts.*` tools in the MCP router (`src/router/mod.rs`), defining their input schemas and ensuring they are discoverable via `tools/list`.
-   **Implementation Stubs**: Created `EncoreTools` struct with "Not Implemented" stubs for all tools (`env.check`, `parse`, `meta`, `run.start`, `run.stop`, `logs.stream`).
-   **Test Integration**: Updated `mcp_contract`, `mcp_tools_test`, `stale_lease_test`, and `verify_test` to instantiate and pass `EncoreTools` to the `Router`.
-   **Dependency Preparation**: Added `encore-tsparser`, `encore-supervisor`, and `encore-runtime-core` to `Cargo.toml`.
    -   *Note*: These dependencies are currently commented out to avoid `swc_common` vs `serde` version conflicts. Full dependency resolution is deferred to the next phase (PR 011 or later) to allow for incremental integration.

## Artifacts
-   `src/tools/encore_ts/`
-   `src/router/mod.rs` (Updated)
-   `tests/` (Updated)

## Verification
-   `make check` passes.
-   `tools/list` correctly reports `encore.ts.*` tools.
-   Calling `encore.ts.*` tools returns a "Not Implemented" error (as expected).
