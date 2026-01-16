# Change 015: Verify Encore and Run Tools

This change ensures that `encore.ts` and `run` tools are robust, verified, and behave as specified.

## Changes
- **Parsed Stability**: Validated `encore.ts.parse` against golden file.
- **Error Handling**: `encore.ts.meta` now correctly fails on errors.
- **Environment**: `encore.ts.env.check` detects missing Node.js and Encore properly.
- **Idempotency**: `encore.ts.run.start` returns consistent Run IDs for same arguments.
- **Persistence**: `state.json` and `logs.ndjson` are deterministically created.
- **Log Replay**: `logs.stream` works for stopped processes by reading from disk.

## Verification
A new test suite `tests/verify_encore_run.rs` was added.
Tests cover:
- `test_parse_golden_stable`
- `test_meta_error_handling`
- `test_env_check_present` / `_missing_node`
- `test_run_idempotency_determinism_and_logs`
- `test_error_codes`

## Usage
To verify:
```bash
cargo test --test verify_encore_run
```
