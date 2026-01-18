# Refactor: AxiomRegent Supervision Wrapper (Full Implementation)

## Goal Description
Implement a complete supervision wrapper in `axiomregent`, including process locking, state management, and robust process control.

## Proposed Changes

### [axiomregent]

#### [MODIFY] [Cargo.toml](file:///Users/bart/Dev/axiomregent/Cargo.toml)
- Add `fs2` (for file locking).
- Add `libc` (for unix signals).

#### [NEW] [src/supervisor/lock.rs](file:///Users/bart/Dev/axiomregent/src/supervisor/lock.rs)
- Implement `FileLock` to ensure single supervisor instance.

#### [NEW] [src/supervisor/state.rs](file:///Users/bart/Dev/axiomregent/src/supervisor/state.rs)
- Define `State` enum (Stopped, Starting, Healthy, Unhealthy).
- Define `SupervisorStatus` struct.

#### [NEW] [src/supervisor/process.rs](file:///Users/bart/Dev/axiomregent/src/supervisor/process.rs)
- Move `Process` struct logic here.
- Implement `kill_gracefully` using `libc` (SIGINT -> SIGTERM -> SIGKILL).

#### [MODIFY] [src/supervisor.rs] -> [src/supervisor/mod.rs](file:///Users/bart/Dev/axiomregent/src/supervisor/mod.rs)
- Convert file to module.
- Integrate `lock`, `state`, `process`.
- Maintain `ReadinessProbe` logic.

## Verification Plan
- `tests/verify_readiness.rs` should continue to pass.
- Verify locking behavior (optional manual check or new test).
