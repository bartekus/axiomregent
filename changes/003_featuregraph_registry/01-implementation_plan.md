# PR 003: FeatureGraph Registry

## Goal Description
Implement the FeatureGraph Registry to manage feature flags and specifications within the MCP server. This involves updating the `featuregraph` crate to correctly parse and represent all metadata in `spec/features.yaml`, and exposing this data via MCP tools (`features.overview`, `features.locate`).

## User Review Required
> [!NOTE]
> The `FeatureNode` struct in `graph.rs` will be expanded to include `owner`, `group`, `depends_on`, `title`, and `governance`. This is a non-breaking change but aligns the code with the spec.

## Proposed Changes

### FeatureGraph Crate
Alignment of the data model with the specification.

#### [MODIFY] [crates/featuregraph/src/graph.rs](file:///Users/bart/Dev/axiomregent/crates/featuregraph/src/graph.rs)
- Add fields to `FeatureNode`: `title`, `owner`, `group`, `depends_on`, `governance`.
- Ensure JSON/Serde compatibility.

#### [MODIFY] [crates/featuregraph/src/scanner.rs](file:///Users/bart/Dev/axiomregent/crates/featuregraph/src/scanner.rs)
- Update `FeatureEntry` to capture all fields from `features.yaml`.
- Populate `FeatureNode` correctly during scanning.

#### [NEW] [crates/featuregraph/src/tools.rs](file:///Users/bart/Dev/axiomregent/crates/featuregraph/src/tools.rs)
- Implement `FeatureGraphTools` struct.
- Implement `features_overview` and `features_locate` methods.

### MCP Router
Integration of the new tools.

#### [MODIFY] [src/router/mod.rs](file:///Users/bart/Dev/axiomregent/src/router/mod.rs)
- Register `FeatureGraphTools` in `Router` struct.
- Add `features.*` tools to `tools/list` and `tools/call`.

#### [MODIFY] [src/main.rs](file:///Users/bart/Dev/axiomregent/src/main.rs)
- Instantiate `FeatureGraphTools`.

### Specs
Update status.

#### [MODIFY] [spec/features.yaml](file:///Users/bart/Dev/axiomregent/spec/features.yaml)
- Mark `FEATUREGRAPH_REGISTRY` as `implementation: in-progress` (or done).
- Update `tests` entry for FeatureGraph.

## Verification Plan

### Automated Tests
- Run `make check` to ensure compilation.
- Run `cargo test -p featuregraph` to verify scanner and graph logic.
- Create a new integration test `tests/mcp_featuregraph_test.rs` to verify MCP tools.

### Manual Verification
- Use `mcp-inspector` or `curl` to call `features.overview` and valid the output matches `features.yaml`.
