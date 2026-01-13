# PR 001 Walkthrough: Core Router

## Overview
Reconstructed the core router infrastructure. The server now supports the MCP JSON-RPC protocol (`initialize`, `tools/list`, `tools/call`) but has a limited set of tools (Registration and Mounts only).

## Changes
- **Pruned** `src/main.rs`: Removed all tool initializations (Snapshot, Feature, Xray, Antigravity) except the core `Resolver` and `MountRegistry`.
- **Pruned** `src/router/mod.rs`: Removed all tool handling logic except `resolve_mcp`, `list_mounts`, and `get_capabilities`.
- **Updated** `src/lib.rs`: Disabled exports for unimplemented modules.

## Verification
### 1. Compilation
`cargo check` passes with clean output (except intentionally unused warnings if any).

### 2. Initialization
Input:
```json
{"jsonrpc":"2.0","method":"initialize","id":1}
```
Output:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "protocolVersion": "2024-11-05",
    "capabilities": { ... },
    "serverInfo": { "name": "mcp", "version": "0.1.0" }
  },
  "id": 1
}
```

### 3. Tool Listing
Input:
```json
{"jsonrpc":"2.0","method":"tools/list","id":2}
```
Output confirms only core tools:
- `resolve_mcp`
- `list_mounts`
- `get_capabilities`
