# Walkthrough - Change 017

## Overview
This change implements the foundational "Lock & State" layer of the Encore Supervisor. It ensures that only one instance of the supervisor can manage a given repository at a time.

## Changes

### Implementation Details

#### 1. Locking (`encore::supervisor::lock`)
Implemented `RepoLock` using `fs2`:
- Uses OS-level file locking (`flock` on Unix).
- Guarantees single-instance execution per repository.
- Automatically releases lock when the `RepoLock` struct is dropped.

#### 2. State Model (`encore::supervisor::state`)
Refined the data structures:
- `SupervisorStatus` struct ready for MCP `encore.status` tool.
- Enums for `State` and `Ownership` with `Display` implementations.

#### 3. Dependencies
- Added `fs2` for locking.
- Added `tempfile` for reliable unit testing.

## Verification
- Unit tests in `lock_tests.rs` pass, verifying:
    - Lock acquisition success.
    - Exclusive locking (second attempt fails).
    - Lock release on drop.
