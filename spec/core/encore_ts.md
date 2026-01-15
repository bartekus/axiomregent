# Feature: Encore TypeScript Toolchain

> [!IMPORTANT]
> This feature provides deep integration with Encore.ts applications, enabling the AxiomRegent agent to understand, run, and verify Encore applications directly.

## Goals
- **Environment Management**: Detect and validate the Encore TS environment.
- **Static Analysis**: Parse `encore.app` metadata to understand services and endpoints.
- **Runtime Control**: Start, stop, and monitor Encore applications in development mode.
- **Log Streaming**: Stream application logs for debugging and verification.

## Architecture

The integration connects the MCP Router to the Encore CLI and runtime. It treats an Encore application as a managed resource that can be inspected and controlled.

### Components

1.  **EncoreTools**: The main entry point for MCP tool calls.
2.  **Env**: Responsible for checking `encore` CLI availability and version.
3.  **Parse**: Bridges `encore-tsparser` to extract application metadata (services, APIs).
4.  **Run**: Manages the `encore run` process lifecycle (start, stop, status).
5.  **Logs**: Handles streaming of structured logs from the running application.

## Tools

### `encore.ts.env.check`
Checks if the `encore` CLI is installed and returns version information.

### `encore.ts.parse`
Parses the Encore application at the specified root and returns a `MetaSnapshot` containing services and API definitions.

### `encore.ts.run.start` / `encore.ts.run.stop`
Controls the `encore run` process. Only one instance per workspace is supported.

### `encore.ts.logs.stream`
Streams logs from the running application, supporting `offset` for polling.
