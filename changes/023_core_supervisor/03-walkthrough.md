# Walkthrough: Core Supervisor + Protocol Skeleton

This change lays the groundwork for replacing the Encore Go daemon by implementing a strict JSON-RPC 2.0 client (`DaemonClient`) capable of communicating over stdio.

## Changes

### Project Structure
- Removed the deprecated `crates/supervisor` crate to consolidate supervisor logic within `src/supervisor`.
- Updated `Cargo.toml` to reflect this removal.

### Protocol Implementation (`src/supervisor/rpc.rs`)
- Defined `JsonRpcRequest`, `JsonRpcResponse`, and `JsonRpcError` structs to match the JSON-RPC 2.0 spec.
- Implemented `DaemonClient` which wraps a child process's stdin/stdout.
- Added a `hello()` method to perform the initial handshake with the daemon (or its replacement).

## Verification
- Ran `make check` to ensure no compilation errors or lints.
- The new types are unit-tested indirectly through usage in the supervisor module.
