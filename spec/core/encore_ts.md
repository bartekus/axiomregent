# Feature: Encore TypeScript Toolchain

> [!IMPORTANT]
> This feature provides deep integration with Encore.ts applications, enabling the AxiomRegent agent to parse, compile, and run Encore applications directly without the Encore Go daemon.

## Goals
- **Environment Management**: Detect and validate `node`, `npm`, and `tsparser-encore`.
- **Parsing**: Directly parse `encore.app` metadata using `tsparser-encore` (Rust).
- **Compilation**: Compile TS code and bundle dependencies.
- **Runtime**: Execute the application using `supervisor-encore` with full process lifecycle control.
- **Log Streaming**: Capture and stream logs from the supervised process.

## Architecture

The integration replaces the opaque `encore run` command with a transparent, componentized pipeline:

1.  **Parse (`tsparser-encore`)**: Extract metadata (`MetaSnapshot`) from source.
2.  **Compile (`tsparser-encore`)**: Transpile TS -> JS and generate artifacts.
3.  **Config Generation**: Produce `supervisor.config.json` and `infra.config.json` from compile outputs.
4.  **Execution (`supervisor-encore`)**: Spawn the supervisor with generated configs to manage service processes.

### Components

1.  **EncoreTools**: The router-facing MCP tool implementation (`src/tools/encore_ts/tools.rs`).
2.  **TsParserClient**: Stdio JSON-RPC client for communicating with `tsparser-encore` binary.
3.  **Supervisor**: The `encore-supervisor` crate renamed/aliased to `supervisor-encore` binary.

## Tools

### `encore.ts.env.check`
Checks availability and versions of:
- `node`, `npm`
- `tsparser-encore` (internal binary)
- `supervisor-encore` (internal binary)

### `encore.ts.parse`
Invokes `tsparser-encore parse`.
- Input: App root path.
- Output: Base64-encoded `MetaSnapshot` (protobuf).

### `encore.ts.codegen`
Invokes `tsparser-encore gen-user-facing`.
- Generates `encore.gen/**` clients and types.

### `encore.ts.compile`
Invokes `tsparser-encore compile`.
- Input: App root, debug mode.
- Output: `CompileResult` containing `CmdSpec` (entrypoints, env vars).
- Artifacts: `.encore/build/**`.

### `encore.ts.run.start`
Orchestrates the run sequence:
1.  Calls `encore.ts.compile`.
2.  Generates `supervisor.config.json` & `infra.config.json` in `.axiomregent/runs/<id>`.
3.  Spawns `supervisor-encore` pointing to these configs.
4.  Captures stdout/stderr into an in-memory ring buffer.
- Returns: `run_id`.

### `encore.ts.run.stop`
Terminates the supervised process group by `run_id`.

### `encore.ts.logs.stream`
Streams logs from the in-memory buffer for a given `run_id`.
