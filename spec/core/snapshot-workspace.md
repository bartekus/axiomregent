# Snapshot & Workspace Tools

**Feature ID**: `MCP_SNAPSHOT_WORKSPACE`
**Implementation**: `src/snapshot/tools.rs`, `src/workspace/tools.rs`

## Overview
These tools provide the fundamental "physics" for interacting with the repository state. They distinguish between **Snapshot Mode** (immutable, virtual) and **Worktree Mode** (mutable, live).

## Coherence Models

### Hybrid Contract
The system operates in two distinct modes:
1.  **Worktree Mode (`worktree`)**: Live view of the filesystem. Coherence is managed via **leases**.
2.  **Snapshot Mode (`snapshot`)**: Immutable view of a captured state. Coherence is managed via **snapshot IDs** (content-addressed).

### Fingerprint
A fingerprint uniquely identifies the state of the repo's HEAD, Index, and Working Tree.
- **Structure**: `{ "head_oid": "...", "index_oid": "...", "status_hash": "..." }`
- **Serialization**: Canonical JSON (lexicographically sorted keys, no whitespace).

### Lease Semantics
- **Issuance**: Issued by `worktree`-mode reads or writes.
- **Validation**: Every `worktree`-mode request with a `lease_id` validates it against current live fingerprint.
- **Stale Lease**: Returns `STALE_LEASE` error if fingerprint differs. Client must retry.

## Schema Definitions
The authoritative schemas for these tools are located in `spec/core/schemas/`.

### Snapshot Tools
Operations on immutable snapshots or reading from the worktree.

- **`snapshot.create`**: Create a new snapshot from the current worktree state.
    - Captures specific paths or lease-touched paths.
- **`snapshot.list`**: List files in a snapshot or worktree.
    - **Mode `worktree`**: Lists live files, updates lease.
    - **Mode `snapshot`**: Lists files from manifest.
- **`snapshot.file`**: Read file content.
- **`snapshot.grep`**: Search for patterns.
    - Deterministic candidate walk (lexicographic).
- **`snapshot.info`**: Get metadata and fingerprints.
- **`snapshot.changes`**: partial diff/changeset between snapshots.
- **`snapshot.diff`**: Detailed unified diffs.
- **`snapshot.export`**: Export snapshot as a bundle (tarball).

### Workspace Tools
Operations that mutate the live worktree.

- **`workspace.write_file`**: Write content to a file.
    - Rejects `..` traversal and absolute paths.
- **`workspace.delete`**: Delete a file.
- **`workspace.apply_patch`**: Apply a patch to the worktree (or virtually to a snapshot).
    - **Worktree**: Validates lease, returns new `fingerprint` + `lease_id`.
    - **Snapshot**: Updates manifest, returns new `snapshot_id`.
    - **Strictness**: Context matching is byte-for-byte.

## Mode Semantics
- **Snapshot Mode**: Operations are performed against a specific `snapshot_id`. Write operations return a *new* `snapshot_id` without modifying disk.
- **Worktree Mode**: Operations are performed directly on the filesystem. Requires a valid `lease_id` for writes to ensure exclusive access.

## Schema Safety Rules
To ensure hybrid coherence and cache safety, the following rules are enforced on schemas:

### Branching Correctness
Hybrid tools (supporting both modes) MUST define success response as `oneOf` with two branches:
1.  **Immutable Branch** (`snapshot` mode):
    - MUST include `cache_hint: "immutable"`.
2.  **Worktree Branch** (`worktree` mode):
    - MUST include `cache_hint: "until_dirty"`.
    - MUST include `lease_id` (string) and `fingerprint` (object).

### Error Enums
- The schema MUST include `STALE_LEASE` in the error code enum.

### Runtime Enforcement
- Implementations MUST verify at runtime that the returned `cache_hint` matches the expectation for the active branch.
