# Implementation Plan - Change 017

## Goal
Implement the core locking mechanism and state definitions for the Encore Supervisor.

## Requirements
1.  **Locking**: Implement `RepoLock` using `fs2` for exclusive OS-level locking.
2.  **State**: Define `Ownership` and `State` enums.
3.  **Dependencies**: Add `fs2` and `tempfile` to `Cargo.toml`.
4.  **Verification**: Verify locking behavior (exclusion, release-on-drop) with unit tests.

## Proposed Changes
### Core Modules
- [NEW] [lock.rs](file:///Users/bart/Dev/axiomregent/crates/encore/supervisor/src/lock.rs)
- [NEW] [state.rs](file:///Users/bart/Dev/axiomregent/crates/encore/supervisor/src/state.rs)
- [NEW] [lock_tests.rs](file:///Users/bart/Dev/axiomregent/crates/encore/supervisor/src/lock_tests.rs)

### Configuration
- [MODIFY] [Cargo.toml](file:///Users/bart/Dev/axiomregent/crates/encore/supervisor/Cargo.toml) (Add dependencies)
