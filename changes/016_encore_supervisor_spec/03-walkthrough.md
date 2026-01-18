# Walkthrough - Change 016

## Overview
This change introduces the core specification for the Encore Daemon Supervisor. It defines the "contract" that subsequent PRs will implement.

## Changes

### Specification
The new spec `spec/core/encore-supervisor.md` defines:
- **State Machine**: `Stopped` -> `Starting` -> `Healthy` / `Unhealthy` -> `Fatal`.
- **Ownership**: `Managed` vs `External`.
- **Locking**: Strict OS-level locking requirements.
- **Readiness**: Layered L1/L2/L3 checks.

### Acceptance Criteria
The golden file `tests/golden/encore_supervisor_trace.golden` establishes the log prefixes that verifying tests will look for.

```text
[encore-sup] INITIALIZING
[encore-sup] ACQUIRING_LOCK path=...
...
```

## Verification Checks
- [x] Spec file created.
- [x] Golden file created.
