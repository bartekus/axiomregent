# Plan: Local Emulator Daemon MVP (PR-2)

## Objective
Implement the core RPC methods required to interact with the Encore Local Emulator Daemon (`daemon.hello`, `infra.ensure`, `daemon.shutdown`) and verify their correctness.

## Changes

1.  **RPC Client (`src/supervisor/rpc.rs`)**:
    *   Add `ensure(infra_config: Value) -> Result<InfraStartResponse>`.
    *   Add `shutdown() -> Result<()>`.
    *   Add `InfraStartResponse` struct to parse the result.

2.  **Supervisor Integration (`src/supervisor/mod.rs`)**:
    *   Integrate `DaemonClient` into the `Supervisor` struct.
    *   Update `Supervisor::run` to:
        *   Spawn the process.
        *   Attach `DaemonClient`.
        *   Call `hello()`.
        *   (Optionally) Call `ensure()` if configuration is available (for MVP, maybe just exposing the method is enough, or a basic test).

3.  **Verification**:
    *   Unit tests for `ensure` and `shutdown` serialization/deserialization.
    *   End-to-end verification (requires a mock daemon or the actual one).

## Context
The `infra.ensure` call is critical as it tells the daemon what resources to provision. The response contains the `runtime_config` needed for the app.
