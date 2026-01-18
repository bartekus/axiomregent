# Implementation Plan - Change 018

## Goal
Implement robust process management for the Encore daemon, including process group handling and safe shutdown guarantees.

## Requirements
1.  **Process Group**: Use platform-specific logic (setsid/setpgid) to ensure the daemon and its children can be killed together.
2.  **ChildGuard**: A struct that wraps the child process and ensures it is killed if the guard is dropped.
3.  **Signal Handling**: correct propagation of SIGTERM/SIGKILL.
4.  **Verification**: Test strict cleanup on drop.

## Proposed Changes
### Core Modules
- [NEW] [process.rs](file:///Users/bart/Dev/axiomregent/crates/encore/supervisor/src/process.rs)
- [NEW] [process_tests.rs](file:///Users/bart/Dev/axiomregent/crates/encore/supervisor/src/process_tests.rs)

### Modifications
- [MODIFY] [lib.rs](file:///Users/bart/Dev/axiomregent/crates/encore/supervisor/src/lib.rs) (Expose `process` module)
