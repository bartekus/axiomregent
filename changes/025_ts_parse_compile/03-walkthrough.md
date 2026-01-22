# Walkthrough: TS Parse & Compile

## Changes
- **`src/tools/encore_ts/client.rs`**: Created client for `tsparser-encore` binary communication via stdin/stdout.
- **`src/tools/encore_ts/tools.rs`**: Implemented `EncoreTools` with `parse`, `codegen`, `compile` methods.
- **`crates/encore/tsparser/src/bin/tsparser-encore.rs`**: Fixed stdio lock issue in tsparser binary.

## Verification
- Verified `encore.ts.parse` successfully returns base64 encoded metadata from fixture app.
- Verified `make check` passes all unit tests.
