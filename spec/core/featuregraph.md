# Feature Graph Registry & Lookup

**Feature ID**: `FEATUREGRAPH_REGISTRY`
**Implementation**: `crates/featuregraph`

## Overview
The Feature Graph allows the system to reason about the repository's capabilities and dependencies. It scans the codebase to build a graph of features, specs, and their relationships.

## Key Concepts
- **Feature ID**: A unique UpperCamelCase identifier for a capability (e.g., `McpRouter`, `GovernanceEngine`).
- **Spec Path**: The markdown file defining the feature's contract.
- **Traceability**: Linkage between `Feature` tags in source code and the registry.

## Tools

### `features.overview`
- **Description**: Returns the full graph of features, including their status, identified implementation files, and test coverage.
- **Scopes**: Can run on `snapshot_id` or `worktree`.

### `features.locate`
- **Description**: Finds the definition or implementation of a feature.
- **Selectors**:
  - `feature_id`: Find by ID.
  - `spec_path`: Find by spec file.
  - `file_path`: Find which feature owns a specific file.

### `features.impact`
- **Description**: Calculates the semantic impact of a set of changed files.
- **Logic**:
  1. Identifies which features own the changed files.
  2. Traverses the dependency graph to find downstream affected features.
  3. Returns a risk profile (e.g., "Critical" if core features are touched).
