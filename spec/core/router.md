# Core MCP Router & Dispatch

**Feature ID**: `MCP_ROUTER`
**Implementation**: `src/router/mod.rs`

## Overview
The Core MCP Router is the central entry point for all JSON-RPC requests in `axiomregent`. It handles protocol negotiation, method dispatch, and error mapping.

## Security Boundaries
1.  **Path Traversal**: All file access requests MUST be validated to be within the allowed `root` path(s).
2.  **Read-Only**: Unless explicitly authorized (e.g. specialized tools), default tools should be read-only or strictly scoped.

## Methods

### `initialize`
- **Description**: Handshakes with the client, returning server capabilities and version info.
- **Protocol**: JSON-RPC 2.0
- **Transport**: Standard Input/Output (stdio).
- **Framing**: Line-delimited JSON messages.

### `tools/list`
- **Description**: Enumerates all available tools registered in the system.
- **Returns**: A list of tool definitions including names, descriptions, and input schemas.

### `tools/call`
- **Description**: Executes a specific tool by name.
- **Parameters**:
  - `name`: String (required)
  - `arguments`: Object (required)
- **Error Handling**: Maps internal `AxiomRegentError` types to JSON-RPC error codes.
    - `ToolNotFound`: If `tools/call` requests unknown tool.
    - `InvalidArgs`: If arguments do not match schema.
    - `SecurityViolation`: If path is outside allowed root.

## Error Codes
The router enforces standard error codes:
- `NOT_FOUND`
- `INVALID_ARGUMENT`
- `REPO_CHANGED`
- `PERMISSION_DENIED`
- `TOO_LARGE`
- `INTERNAL`
