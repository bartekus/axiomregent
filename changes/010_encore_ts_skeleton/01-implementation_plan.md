# PR 010: Encore TS Integration Skeleton

## Goal
Implement the foundational skeleton for the Encore TS toolchain integration.

## Proposed Changes
- **Directory Structure**: Add `src/tools/encore_ts/` with submodules (`env`, `parse`, `projection`, `run`, `schemas`, `state`, `tools`).
- **Tool Registration**: Register `encore.ts.*` tools in the MCP router (`src/router/mod.rs`).
- **Implementation Stubs**: Create `EncoreTools` struct with "Not Implemented" stubs.
- **Test Integration**: Update `tests/` to integrate `EncoreTools`.
- **Dependency Preparation**: Add `encore-tsparser` and related crates to `Cargo.toml`.

## Verification Plan
- `make check` to pass.
- `tools/list` to report `encore.ts.*` tools.
- Calling usage of stubbed tools returns "Not Implemented".
