# Build Fixes Walkthrough

This document summarizes the changes made to resolve build issues in `swc_common` and `axiomregent` crates.

## Resolution Summary

The goal was to fix all compilation errors and `clippy` lints to ensure `make check` passes.

### `swc_common` Fixes

Several lints were addressed in the `swc_common` crate:

1.  **`eq.rs` & `pos.rs`**: Resolved `clippy::needless_lifetimes` by eliding lifetimes in `impl` blocks.
2.  **`errors/emitter.rs`**: Fixed `clippy::unnecessary_get_then_check` by using `contains_key`.
3.  **`source_map.rs`**: Addressed `clippy::double_ended_iterator_last` and `clippy::legacy_numeric_constants`.
4.  **`syntax_pos/hygiene.rs`**: Fixed `clippy::explicit_auto_deref`.
5.  **`comments.rs`**: Resolved `clippy::mismatched_lifetime_syntaxes`.
6.  **`rustc_data_structures/stable_hasher.rs`**:
    *   Fixed `clippy::unnecessary_transmutes` by using `f32::to_bits()` / `f64::to_bits()`.
    *   Added `#![allow(dead_code)]` to suppress warnings for unused vendored code.

### `axiomregent` Fixes

The `axiomregent` build had issues related to `encore_ts` integration:

1.  **Parser Initialization**:
    *   Updated `PassOneParser::new` call in `src/tools/encore_ts/parse.rs` to include missing arguments (`file_set`, `type_checker`, registry).
    *   Fixed unsatisfied imports (`ParseContext`, `Parser`).
2.  **Clippy Lints**:
    *   Fixed `clippy::borrow_deref_ref` in `parse.rs`.
    *   Fixed `clippy::lines_filter_map_ok` (infinite loop potential) in `run.rs` by using `.map_while(Result::ok)`.
    *   Fixed `clippy::collapsible_if` and `manual_flatten` in `run.rs`.
    *   Fixed `clippy::new_without_default` in `state.rs`.
3.  **Panic Fixes**:
    *   Wrapped parsing logic in `swc_common::GLOBALS.set` and `swc_common::errors::HANDLER.set` to prevent panics during `SourceMap` operations in tests.
4.  **Integration Test**:
    *   Updated `tests/encore_integration.rs` to match the new `encore_app` fixture structure (expecting `exampleService` instead of `greeting`).

## Verification Results

Verified that `make check` passes cleanly, including all unit and integration tests.

```bash
make check
```

All 13 unit tests passed, as well as the 14+ integration tests.
