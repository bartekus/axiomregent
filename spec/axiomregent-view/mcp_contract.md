# MCP Tool Contract

This document defines the available MCP tools, their contracts, and discovery mechanisms.

## Discovery
Clients **must** call `tools/list` on connection to discover available tools.
The UI should only expose features for tools present in the `tools/list` response.

## Universal Schemas
- **Error Schema**: `AxiomRegentError` (Code, Message)

## Tool Table

| Tool Name | Summary | Request Schema | Response Schema | Safety Tier |
| :--- | :--- | :--- | :--- | :--- |
| `antigravity.execute` | Execute a changeset | `ExecuteRequest` | `ChangesetStatusV1` | Tier 3 (Destructive) |
| `antigravity.propose` | Propose a change | `ProposeRequest` | `ChangesetMetaV1` | Tier 2 (Mutation) |
| `antigravity.verify` | Verify a changeset | `VerifyRequest` | `ChangesetStatusV1` | Tier 1 (Read/Safe) |
| `encore.ts.env.check` | Check Encore env | `{}` | `EnvCheckResult` | Tier 1 (Read) |
| `encore.ts.logs.stream` | Stream app logs | `StreamLogsRequest` | `LogStreamResponse` | Tier 1 (Read) |
| `encore.ts.meta` | Get app metadata | `EncoreMetaRequest` | `MetaSnapshotV1` | Tier 1 (Read) |
| `encore.ts.parse` | Parse app source | `EncoreParseRequest` | `MetaSnapshotV1` | Tier 1 (Read) |
| `encore.ts.run.start` | Start Encore app | `EncoreRunStartRequest` | `jsonal` (Run ID) | Tier 3 (Execution) |
| `encore.ts.run.stop` | Stop Encore app | `EncoreRunStopRequest` | `void` | Tier 3 (Execution) |
| `features.locate` | Locate feature | `LocateFeatureRequest` | `FeatureLocation` | Tier 1 (Read) |
| `features.overview` | Get feature graph | `FeatureOverviewRequest` | `FeatureGraph` | Tier 1 (Read) |
| `gov.drift` | Check for drift | `GovDriftRequest` | `DriftReport` | Tier 1 (Read) |
| `gov.preflight` | Check policy | `GovPreflightRequest` | `PreflightReport` | Tier 1 (Read) |
| `resolve_mcp` | Resolve server | `ResolveRequest` | `ResolveResult` | Tier 1 (Read) |
| `run.execute` | Execute skill | `RunExecuteRequest` | `RunStatus` | Tier 3 (Execution) |
| `run.logs` | Get run logs | `RunLogsRequest` | `String` (Logs) | Tier 1 (Read) |
| `run.status` | Get run status | `RunStatusRequest` | `RunStatus` | Tier 1 (Read) |
| `snapshot.changes` | List changes | `SnapshotChangesRequest` | `path[]` | Tier 1 (Read) |
| `snapshot.create` | Create snapshot | `SnapshotCreateRequest` | `snapshot_id` | Tier 2 (Mutation) |
| `snapshot.diff` | Gen unified diff | `SnapshotDiffRequest` | `String` (Diff) | Tier 1 (Read) |
| `snapshot.export` | Export tarball | `SnapshotExportRequest` | `tarball_bytes` | Tier 1 (Read) |
| `snapshot.grep` | Search patterns | `SnapshotGrepRequest` | `Match[]` | Tier 1 (Read) |
| `snapshot.info` | Get snap info | `SnapshotInfoRequest` | `SnapshotInfo` | Tier 1 (Read) |
| `snapshot.list` | List files | `SnapshotListRequest` | `FileEntry[]` | Tier 1 (Read) |
| `snapshot.read` | Read content | `SnapshotReadRequest` | `String` (Content) | Tier 1 (Read) |
| `workspace.apply_patch`| Apply patch | `ApplyPatchRequest` | `void` | Tier 3 (Destructive) |
| `workspace.delete` | Delete file/dir | `DeleteRequest` | `void` | Tier 3 (Destructive) |
| `workspace.write_file` | Write content | `WriteFileRequest` | `void` | Tier 3 (Destructive) |
| `xray.scan` | Scan repo index | `XrayScanRequest` | `XrayIndex` | Tier 1 (Read) |

## JSON Examples

### Antigravity Propose
Request:
```json
{
  "name": "antigravity.propose",
  "arguments": {
    "repo_root": "/abs/path",
    "subject": "Refactor router",
    "repo_key": "axiomregent",
    "goal": "Improve modularity",
    "tasks": [],
    "base_state": "HEAD"
  }
}
```

### Antigravity Execute
Request:
```json
{
  "name": "antigravity.execute",
  "arguments": {
    "repo_root": "/abs/path",
    "changeset_id": "014_refactor_router"
  }
}
```
Response (ChangesetStatusV1):
```json
{
  "schema_version": "v1",
  "state": "executed",
  "validation": { "state": "valid", "checks": [] },
  "execution": { "state": "completed", "steps_completed": 5, "error": null, "log": [] },
  "verification": {
      "last_run": {
          "profile": "default",
          "outcome": "success"
      }
  }
}
```
