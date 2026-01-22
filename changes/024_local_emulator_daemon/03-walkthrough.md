# Walkthrough: Local Emulator Daemon MVP

This change implements the minimal viable protocol for interacting with the Encore Local Emulator Daemon, allowing AxiomRegent to request infrastructure provisioning and manage the daemon lifecycle.

## Changes

### RPC Client Enhancements (`src/supervisor/rpc.rs`)
- Added `ensure` method to call `infra.ensure`, allowing the supervisor to request infrastructure provisioning.
- Added `shutdown` method to call `daemon.shutdown`.
- Defined `InfraStartResponse` to capture the `runtime_config` returned by the daemon.

### Supervisor Integration (`src/supervisor/mod.rs`)
- Wired the `DaemonClient` into the main `Supervisor` struct.
- Updated the `run` loop to initialize the client and perform the `hello` handshake upon process start.

## Verification
- Verified that `ensure` calls are correctly serialized and sent to the child process.
- Verified that the supervisor can parse the `InfraStartResponse` correctly.
