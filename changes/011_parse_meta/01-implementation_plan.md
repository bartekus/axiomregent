# Implementation Plan - 011 Parse & Meta

## Goal
Implement the parsing and metadata extraction logic for the Encore TypeScript toolchain (`encore-tsparser`). This builds upon the skeleton set up in change 010.

## User Review Required
None anticipated yet.

## Proposed Changes

### Configuration
- Create `changes/011_parse_meta` directory to track this change.

### `crates/encore/tsparser`
- [x] Fix compilation errors in `src/runtimeresolve/node.rs` (mismatched types, missing annotations).
- [x] Verify Implementation
- [x] Implement/Refine `src/legacymeta`: Logic to extract metadata references. (Verified existing logic passes tests)
- [x] Ensure `src/lib.rs` exports necessary modules.

## Verification Plan

### Automated Tests
- [x] Run `cargo check -p encore-tsparser` to ensure compilation.
- [x] Run `cargo test -p encore-tsparser` to run unit tests.
- [x] Verify `test_parse` utility works as expected for simple TypeScript snippets.
