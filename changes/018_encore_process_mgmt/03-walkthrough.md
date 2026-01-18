# Walkthrough - Change 018

## Overview
This change implements the process management layer for the Encore Supervisor. It provides safe wrappers for spawning child processes, handling process groups, and ensuring termination.

## Changes

### 1. Process Guard (`encore::supervisor::process`)
- `ChildGuard`: A wrapper that kills the child process when dropped.
- `spawn()`: Configures `tokio::process::Command` with:
    - Piped stdout/stderr.
    - New process group (`setsid` on Unix).
    - Environment variable injection.

### 2. Signal Handling
- Implements `kill()` using `libc::kill` to target the *process group* (negative PID).
- Falls back to `SIGKILL` if `SIGTERM` exceeds 5s timeout.

### 3. Testing
- `process_tests.rs` verifies:
    - Spawning works.
    - `kill()` reliably terminates the process.
    - Process group cleanup (basic verification).
    - Drop guard cleanup.

## Verification
- Unit tests passed on macOS.
