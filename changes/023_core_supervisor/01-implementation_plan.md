# Plan: Core Supervisor + Protocol Skeleton

## Objective
Implement the foundational JSON-RPC 2.0 protocol skeleton for supervising the Encore "Local Emulator Daemon" and clean up the project structure by removing the redundant `crates/supervisor` crate, favoring the existing `src/supervisor` module.

## Changes
1.  **Project Structure**:
    *   Removed `crates/supervisor` from the workspace and file system.
    *   Updated `Cargo.toml` to remove the supervisor crate dependency.
    *   Updated `src/supervisor/mod.rs` to expose the new `rpc` module.

2.  **Protocol Skeleton (`src/supervisor/rpc.rs`)**:
    *   Implemented `JsonRpcRequest`, `JsonRpcResponse`, `JsonRpcError` structs.
    *   Implemented `DaemonClient` for managing JSON-RPC communication over stdio with a child process.
    *   Added support for sending requests and handling responses with ID correlation.
    *   Implemented a strictly typed `hello` handshake method.

## Verification
*   `make check` ensures compilation and type safety.
*   Existing `src/supervisor` functionality (process management, locking) remains intact.
