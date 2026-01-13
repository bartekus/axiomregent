# PR 001: Core MCP Router & Infrastructure

**Goal**: Establish the foundational infrastructure for the AxiomRegent MCP server. This includes the main entry point, the JSON-RPC router, and core support modules (IO, Config, Resolver).

## User Review Required
> [!IMPORTANT]
> This PR involves **destructive** modification of `src/main.rs` and `src/router/mod.rs` to "reconstruct" their initial state.
> The original full versions will be backed up to `src/main.rs.original` and `src/router/mod.rs.original` to preserve the future code for subsequent PRs.
> Please ensure you do not commit the `.original` files if you are strictly following a clean PR history, OR we can keep them ignored.

## Proposed Changes

### Core Infrastructure
#### [NEW] [src/io](file:///Users/bart/Dev/axiomregent/src/io)
- Files: `mod.rs`, `fs.rs`, `memfs.rs`
- Logic: Filesystem abstractions (Real and Memory).

#### [NEW] [src/util](file:///Users/bart/Dev/axiomregent/src/util)
- Files: `mod.rs`, `paths.rs`, `stable.rs`
- Logic: Path normalization and helper utilities.

#### [NEW] [src/config](file:///Users/bart/Dev/axiomregent/src/config)
- Files: `mod.rs`
- Logic: Configuration loading (Environment, defaults).

#### [NEW] [src/resolver](file:///Users/bart/Dev/axiomregent/src/resolver)
- Files: `mod.rs`, `order.rs`, `alias_map.rs`, `git_remote.rs`, `workspace.rs`
- Logic: Resolving repository names to paths.

#### [NEW] [src/protocol](file:///Users/bart/Dev/axiomregent/src/protocol)
- Files: `mod.rs`, `types.rs`
- Logic: Shared types.

### Generic Router
#### [MODIFY] [src/router/mod.rs](file:///Users/bart/Dev/axiomregent/src/router/mod.rs)
- **Action**: Prune implementation.
- **Content**:
    - Keep `Router` struct shell.
    - Keep `JsonRpcRequest`, `JsonRpcResponse`, `AxiomRegentError`.
    - Keep `initialize`, `tools/list` (filtered), `tools/call` (filtered).
    - **Remove**: Logic for `snapshot.*`, `workspace.*`, `antigravity.*`, `features.*`, `xray.*`.
    - **Keep**: `resolve_mcp`, `list_mounts`, `get_capabilities`.

#### [MODIFY] [src/main.rs](file:///Users/bart/Dev/axiomregent/src/main.rs)
- **Action**: Prune initialization.
- **Content**:
    - Setup Logging & Panic Safety.
    - Setup `Resolver`.
    - Setup `MountRegistry`.
    - Setup `Router` (without extra tools).
    - Run Stdio Loop.

### Library Definition
#### [MODIFY] [src/lib.rs](file:///Users/bart/Dev/axiomregent/src/lib.rs)
- Remove module declarations for future modules (`antigravity_tools`, `feature_tools`, `snapshot`, etc.) if they break compilation, or just leave them if the files exist but are effectively unused/empty.
- *Decision*: We will comment out the modules in `lib.rs` that are not yet "introduced" provided we also ignore the files, or we can just leave `lib.rs` pointing to them but they won't be used.
- *Better Plan*: Reference only the modules in this PR.

## Verification Plan

### Automated Tests
1.  **Compile**: `cargo check --bin axiomregent`
2.  **Test**: `cargo test --lib src/router` (if applicable)

### Manual Verification
1.  **Run**: `cargo run --bin axiomregent`
2.  **Interact**: Send `initialize` JSON-RPC message.
3.  **Interact**: Send `tools/list` and verify only core tools are present.
