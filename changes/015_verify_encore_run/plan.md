# Verification of Encore and Run Tools

## Goal Description
Verify reliability, stability, and correctness of `encore.ts` and `run` tools. This includes stability of parser output, error handling, environment detection, run idempotency, log persistence, and log streaming replay.

## Implementation Details
### Testing
- Created `tests/verify_encore_run.rs` as a comprehensive verification suite.
- Added `tests/golden/encore_parse_snapshot.json` for parser stability.
- Created `tests/fixtures/encore_app_error` for error handling tests.
- Mocked `encore` CLI in `tests/bin/encore` for reliable environment/run testing.

### Tool Updates
- **Parser (`src/tools/encore_ts/parse.rs`)**: Updated to check for and return errors from the SWC handler.
- **Env (`src/tools/encore_ts/env.rs`)**: Added check for `node`, improved version parsing, and graceful handling of missing `encore` binary.
- **State (`src/tools/encore_ts/state.rs`)**: Added `env` field to `RunProcess` for idempotency checks.
- **Run (`src/tools/encore_ts/run.rs`)**: Implemented idempotency (returning existing `run_id`), persisting start state to `state.json`, and capturing logs to `logs.ndjson`.
- **Tools (`src/tools/encore_ts/tools.rs`)**: Implemented `logs_stream` fallback to read from disk if the process is not in memory.

## Verification
Run the verification suite:
```bash
cargo test --test verify_encore_run -- --test-threads=1
```
All 6 tests passed.
