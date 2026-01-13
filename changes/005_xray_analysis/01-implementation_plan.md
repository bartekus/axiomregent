# Implementation Plan - PR 005: Xray Analysis

This plan outlines the steps to implement the **Xray Analysis** feature (ID: `XRAY_ANALYSIS`), which integrates the `xray` crate into the core MCP router, exposing repository scanning capabilities via the `xray.scan` tool.

## User Review Required

> [!NOTE]
> This change exposes `xray.scan` which performs a full repository scan. For large repositories, this might be slow, but it's essential for initial context building.

## Proposed Changes

### `crates/xray`

#### [NEW] [crates/xray/src/tools.rs](file:///Users/bart/Dev/axiomregent/crates/xray/src/tools.rs)
- Implement `XrayTools` struct.
- Implement `xray_scan` method which wraps `scan_target`.
- Define input schema for usually `repo_root` (or `path` relative to it).

#### [MODIFY] [crates/xray/src/lib.rs](file:///Users/bart/Dev/axiomregent/crates/xray/src/lib.rs)
- Export `tools` module.

### Core System

#### [MODIFY] [src/router/mod.rs](file:///Users/bart/Dev/axiomregent/src/router/mod.rs)
- Import `xray::tools::XrayTools`.
- Add `xray_tools` to `Router` struct and `new` method.
- Register `xray.scan` in `tools/list`.
- Dispatch `xray.scan` in `tools/call`.

#### [MODIFY] [src/main.rs](file:///Users/bart/Dev/axiomregent/src/main.rs)
- Initialize `XrayTools` and pass to `Router`.

#### [MODIFY] [src/lib.rs](file:///Users/bart/Dev/axiomregent/src/lib.rs)
- Update if necessary (re-exports).

## Verification Plan

### Automated Tests
- Run `make check` to ensure compilation and linting.
- Add unit tests in `crates/xray/src/tools.rs` to verify tool logic (using mocks or temp dirs).
- Run `cargo test -p xray`.
- Run `cargo test` generally to ensure no regressions.

### Manual Verification
- Use `mcp-inspector` or similar (if available) or `curl` to call `tools/list` and verify `xray.scan` is present.
- Call `xray.scan` on the repo itself and verify the output.
