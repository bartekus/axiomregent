# Fix Encore Test Failures

The goal is to resolve test failures in `verify_encore_run` (concurrency/env issues) and `encore_integration` (parser resolution errors).

## User Review Required

> [!NOTE]
> `verify_encore_run.rs` modifies the global `PATH` environment variable. While `Mutex` is used, a panic in one test can leave the environment in a bad state (empty PATH), causing other tests to fail. The fix introduces an RAII guard to safely restore the environment.

## Proposed Changes

### Tests

#### [MODIFY] [verify_encore_run.rs](file:///Users/bart/Dev/axiomregent/tests/verify_encore_run.rs)
- Introduce `EnvGuard` struct that restores env vars on Drop.
- Update `test_env_check_missing_node` to use `EnvGuard`.
- Add debugging/better assertion for "PATH is empty but check passed" failure (including printing found version).
- Fix `PoisonError` by ensuring locks are released/handled safely (Drop ensures this mostly, but panics poison mutexes. We should maybe use `catch_unwind` or just rely on fix preventing panic).

#### [MODIFY] [tests/encore_integration.rs](file:///Users/bart/Dev/axiomregent/tests/encore_integration.rs)
- Investigate why `swc` fails to find `node_modules`.
- Ensure `node_modules` in `tests/fixtures/encore_app` is correctly structured.
- If necessary, mock resolution or adjust `swc` config if it's a known issue with `node_modules` resolution in `swc` crate.

## Verification Plan

### Automated Tests
- Run `cargo test --test verify_encore_run` to verify fix for env checks.
- Run `cargo test --test encore_integration` to verify parser fixes.
- Run `make check` to ensure no regressions.

### Manual Verification
- None required beyond automated tests.
