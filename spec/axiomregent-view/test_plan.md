# Test Plan for AxiomRegent View

This document defines the verification strategy for `axiomregent-view`.

## 1. Manual Acceptance Checks

### Connection Flow
- [ ] **Launch App**: Open the UI.
- [ ] **Connect**: App should auto-connect to default MCP port (or prompt).
- [ ] **Verify**: Status indicator turns GREEN. Capabilities list populates.

### Discovery Flow
- [ ] **Navigate**: Go to "Tools" tab.
- [ ] **Verify**: List contains at least the **core tools** (`resolve_mcp`, `list_mounts`, `features.overview`) plus any available extensions.
- [ ] **Verify**: Tools are sorted alphabetically.
- [ ] **Verify**: Clicking a tool opens the Runner.

### Execution Flow (Tier 1)
- [ ] **Select**: `resolve_mcp` (or any simple read tool).
- [ ] **Input**: Enter generic arguments (e.g., `name: "axiomregent"`).
- [ ] **Run**: Click "Execute".
- [ ] **Verify**: Output appears in Console. JSON Preview matches Input.

### Changeset Flow (Mocked)
- [ ] **Navigate**: Go to "Changesets".
- [ ] **Verify**: List of mock changesets appears.
- [ ] **Select**: Click a "Verified" changeset.
- [ ] **Verify**: Detail view shows "Verification" panel with success outcome.
- [ ] **Verify**: Timestamp is shown ONLY if present in the mock data.

## 2. Golden Path Workflows

### Scenario 1: Feature Discovery
1. Connect to MCP.
2. Call `tools/list` -> Receive tool list.
3. User filters for "feature".
4. User selects `features.overview`.
5. User enters `repo_root`.
6. UI displays Feature Graph result.

### Scenario 2: Governance Check
1. Connect to MCP.
2. User selects `gov.preflight`.
3. User inputs complex `changed_paths`.
4. UI displays Preflight Report (Safety Tier 2).

## 3. Deterministic Fixture Guidance

To test the UI without a running backend, use a **Mock MCP Server** that serves static fixtures.

### Fixture: `tools_list.json`
Must contain at least the core tools from `mcp_contract.md`. Additional tools are optional.

### Fixture: `changeset_verified.json`
Should return a `ChangesetStatus` with:
- `state`: "executed"
- `validation.state`: "valid"
- `verification`: `{ "last_run": { "outcome": "success", "profile": "default" } }`
- **Note**: Omit `timestamp` for deterministic snapshots.

### Fixture: `changeset_failed.json`
Should return a `ChangesetStatus` with:
- `state`: "failed"
- `verification`: null

## 4. Automated Testing Strategy

### Unit Tests
- **Components**: Render components with fixture data. Assert text presence.
- **State**: Test reducers/stores with canonical JSON actions.

### Integration Tests
- **Mock Service Worker (MSW)**: Intercept generic MCP JSON-RPC calls.
- **Response**: Return deterministic JSON from fixtures.
- **Assert**: Verify UI state matches fixture data.

## 5. Mock MCP Strategy
The UI *must* be capable of running against a "Replay Server" that strictly replays a recorded session.
- No timestamps generated client-side.
- No random IDs generated client-side.
- All "current time" displays must be derived from server data or hidden unless server data exists.
