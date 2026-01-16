# Resolve Build Issues & Fix Encore Integration

## Goal Description
Resolve all remaining `clippy` lints and compilation errors across `swc_common` and `axiomregent` crates to ensure `make check` passes. Additionally, fix the `encore_ts` parser integration by addressing build failures, panics in tests, and aligning test fixtures.

## Proposed Changes

### `swc_common`
Resolve various lints including `clippy::explicit_auto_deref`, `clippy::mismatched_lifetime_syntaxes`, and unused code warnings.

#### [MODIFY] [hygiene.rs](file:///Users/bart/Dev/axiomregent/crates/encore/libs/swc/crates/swc_common/src/syntax_pos/hygiene.rs)
- Remove explicit auto-dereference.

#### [MODIFY] [comments.rs](file:///Users/bart/Dev/axiomregent/crates/encore/libs/swc/crates/swc_common/src/comments.rs)
- Add explicit anonymous lifetimes to fix mismatched revenue syntaxes.

#### [MODIFY] [stable_hasher.rs](file:///Users/bart/Dev/axiomregent/crates/encore/libs/swc/crates/swc_common/src/rustc_data_structures/stable_hasher.rs)
- Replace `transmute` with `to_bits()` for float hashing.
- Suppress unused code warnings.

### `axiomregent`

#### [MODIFY] [parse.rs](file:///Users/bart/Dev/axiomregent/src/tools/encore_ts/parse.rs)
- Fix `PassOneParser::new` arguments.
- Resolve references to `ParseContext` and `Parser`.
- Wrap parsing in `swc_common::GLOBALS.set` and `HANDLER.set` to prevent panics.
- Fix `clippy::borrow_deref_ref`.

#### [MODIFY] [run.rs](file:///Users/bart/Dev/axiomregent/src/tools/encore_ts/run.rs)
- Fix infinite loop risk by using `map_while` instead of `flatten`.
- Fix `clippy::collapsible_if`.

#### [MODIFY] [state.rs](file:///Users/bart/Dev/axiomregent/src/tools/encore_ts/state.rs)
- Implement `Default` for `EncoreState`.

#### [MODIFY] [encore_integration.rs](file:///Users/bart/Dev/axiomregent/tests/encore_integration.rs)
- Update assertions to match `encore_app` fixture content (expecting `exampleService`).

#### [RENAME] [greeting.ts -> encore.service.ts](file:///Users/bart/Dev/axiomregent/tests/fixtures/encore_app/encore.service.ts)
- Rename fixture file to valid Encore service filename.

## Verification Plan

### Automated Tests
- Run `make check` to verify all lints and tests pass.
- Run `cargo test --test encore_integration` specifically to verify the parser fix.
