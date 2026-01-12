# Xray Repository Analysis

**Feature ID**: `XRAY_ANALYSIS`
**Implementation**: `crates/xray`

## Overview
Xray is the deep-scanning engine that builds a content-addressed index of the repository. It identifies languages, modules, definitions, and references to power sematic understanding.

## Tools

### `xray.scan`
- **Description**: Scans a target directory and returns a complete `XrayIndex`.
- **Phases**:
  1.  **Traversal**: Walks the filesystem (respecting gitignore).
  2.  **Indexing**: Parses files to extract symbols and metadata.
  3.  **Digest**: Computes a deterministic hash of the index.
  4.  **Serialization**: Outputs canonical JSON.

## Scan Policy (`spec/xray/scan-policy.md`)
- **Scope**: Scans target recursively.
- **Exclusion**: Ignores dot-directories (`.git`, `.axiomregent`) by default.
- **Determinism**:
    - **LOC Counting**: `str::lines().count()` (logical lines).
    - **Canonical Output**: JSON keys sorted lexicographically.
    - **Stable Hash**: File digests are SHA256 of content.

## Data Model (`XrayIndex`)
Defined in `spec/xray/index-format.md`.

- **Root**: Repository slug/name.
- **Target**: Relative path scanned.
- **Digest**: SHA256 of the *content* of the index (integrity check).
- **Files**: List of `FileNode` objects (sorted by path).
    - `path`: Relative path.
    - `loc`: Logical lines.
    - `size`: Bytes.
    - `language`: Detected language (or "Unknown").
    - `digest`: SHA256 content hash.
