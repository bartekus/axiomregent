# Implementation Plan: TS Run Start Integration

## Goal
Run the compiled Encore TS app using `supervisor-encore` managed by AxiomRegent, replacing `encore run`.

## Proposed Changes

### [MODIFY] `src/tools/encore_ts/tools.rs`
- **`run_start`**: 
    1. Invoke `compile`.
    2. Read `CompileResult`.
    3. Generate `supervisor.config.json` mapping `CmdSpec` to `Proc`.
    4. Generate `infra.config.json`.
    5. Spawn `supervisor-encore` with these configs.
    6. Streaming log capture.
- **`run_stop`**: Kill child process.
- **`logs_stream`**: Return logs from ring buffer.

### [MODIFY] `crates/encore/supervisor`
- Expose `Proc` and config structs to facilitate external config generation.

### [MODIFY] `crates/encore/tsparser`
- Export `CompileResult` and derive `Deserialize` for consumption by tools.

## Verification
- Integration test `test_run_persistence`: Start app, check files exist (configs), check logs, stop.
